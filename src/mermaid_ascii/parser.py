"""Mermaid flowchart parser — hand-rolled recursive descent.

1:1 port of parser.rs logic (adapted from pest to Python).
Parses Mermaid flowchart/graph DSL into the AST types from ast.py.
"""

from __future__ import annotations

import re
from dataclasses import dataclass

from mermaid_ascii.ast import (
    Direction,
    Edge,
    EdgeType,
    Graph,
    Node,
    NodeShape,
    Subgraph,
)

# ─── Tokenizer ───────────────────────────────────────────────────────────────

# Token patterns (order matters for alternation)
_COMMENT_RE = re.compile(r"%%[^\n]*")
_WHITESPACE_RE = re.compile(r"[ \t]+")
_NEWLINE_RE = re.compile(r"\r\n|\n|\r")

# Edge connectors — longer/more specific first
_EDGE_PATTERNS: list[tuple[str, EdgeType]] = [
    ("<-.->", EdgeType.BidirDotted),
    ("<==>", EdgeType.BidirThick),
    ("<-->", EdgeType.BidirArrow),
    ("-.->", EdgeType.DottedArrow),
    ("==>", EdgeType.ThickArrow),
    ("-->", EdgeType.Arrow),
    ("-.-", EdgeType.DottedLine),
    ("===", EdgeType.ThickLine),
    ("---", EdgeType.Line),
]

_NODE_ID_RE = re.compile(r"[a-zA-Z_][a-zA-Z0-9_-]*")
_DIRECTION_RE = re.compile(r"TD|TB|LR|RL|BT")
_BARE_LABEL_RE = re.compile(r"[^\]\)\}\n]+")
_LABEL_TEXT_RE = re.compile(r"[^|\n]+")
_BARE_SUBGRAPH_LABEL_RE = re.compile(r"[^\n]+")


@dataclass
class _Parser:
    """Stateful parser cursor over the input string."""

    src: str
    pos: int = 0

    # ── Primitive helpers ─────────────────────────────────────────────────────

    def eof(self) -> bool:
        return self.pos >= len(self.src)

    def peek(self, s: str) -> bool:
        return self.src.startswith(s, self.pos)

    def consume(self, s: str) -> bool:
        if self.peek(s):
            self.pos += len(s)
            return True
        return False

    def match_re(self, pattern: re.Pattern[str]) -> str | None:
        m = pattern.match(self.src, self.pos)
        if m:
            self.pos = m.end()
            return m.group(0)
        return None

    def skip_ws(self) -> None:
        """Skip spaces/tabs and inline comments (not newlines)."""
        while True:
            m = _WHITESPACE_RE.match(self.src, self.pos)
            if m:
                self.pos = m.end()
                continue
            m = _COMMENT_RE.match(self.src, self.pos)
            if m:
                self.pos = m.end()
                continue
            break

    def skip_ws_and_newlines(self) -> None:
        """Skip spaces, tabs, comments, and newlines."""
        while True:
            m = _WHITESPACE_RE.match(self.src, self.pos)
            if m:
                self.pos = m.end()
                continue
            m = _COMMENT_RE.match(self.src, self.pos)
            if m:
                self.pos = m.end()
                continue
            m = _NEWLINE_RE.match(self.src, self.pos)
            if m:
                self.pos = m.end()
                continue
            break

    def consume_newline(self) -> bool:
        m = _NEWLINE_RE.match(self.src, self.pos)
        if m:
            self.pos = m.end()
            return True
        return False

    # ── Direction ─────────────────────────────────────────────────────────────

    def parse_direction_value(self) -> Direction:
        d = self.match_re(_DIRECTION_RE)
        if d in ("TD", "TB"):
            return Direction.TD
        if d == "LR":
            return Direction.LR
        if d == "RL":
            return Direction.RL
        if d == "BT":
            return Direction.BT
        return Direction.TD

    # ── Header ────────────────────────────────────────────────────────────────

    def try_parse_header(self) -> Direction | None:
        """Try to parse 'graph TD' or 'flowchart LR' header. Returns None if absent."""
        saved = self.pos
        self.skip_ws_and_newlines()
        if self.consume("flowchart") or self.consume("graph"):
            self.skip_ws()
            d = self.parse_direction_value()
            self.skip_ws()
            # Allow optional comment then newline
            self.skip_ws()
            m = _COMMENT_RE.match(self.src, self.pos)
            if m:
                self.pos = m.end()
            self.skip_ws()
            self.consume_newline()
            return d
        self.pos = saved
        return None

    # ── Quoted string ─────────────────────────────────────────────────────────

    def parse_quoted_string(self) -> str:
        """Parse "..." with backslash escaping. Caller must have seen opening quote."""
        assert self.src[self.pos] == '"'
        self.pos += 1  # consume "
        buf: list[str] = []
        while self.pos < len(self.src):
            ch = self.src[self.pos]
            if ch == '"':
                self.pos += 1
                break
            if ch == "\\" and self.pos + 1 < len(self.src):
                nxt = self.src[self.pos + 1]
                if nxt == "n":
                    buf.append("\n")
                elif nxt == '"':
                    buf.append('"')
                elif nxt == "\\":
                    buf.append("\\")
                else:
                    buf.append(nxt)
                self.pos += 2
            else:
                buf.append(ch)
                self.pos += 1
        return "".join(buf)

    # ── Node label (inside shape brackets) ───────────────────────────────────

    def parse_node_label(self) -> str:
        self.skip_ws()
        if self.pos < len(self.src) and self.src[self.pos] == '"':
            return self.parse_quoted_string()
        label = self.match_re(_BARE_LABEL_RE)
        return (label or "").strip()

    # ── Node shape ────────────────────────────────────────────────────────────

    def parse_node_shape(self) -> tuple[NodeShape, str] | None:
        """Try to parse shape brackets. Returns (shape, label) or None."""
        if self.peek("(("):
            self.pos += 2
            label = self.parse_node_label()
            self.consume("))")
            return (NodeShape.Circle, label)
        if self.peek("(") and not self.src.startswith("((", self.pos):
            self.pos += 1
            label = self.parse_node_label()
            self.consume(")")
            return (NodeShape.Rounded, label)
        if self.peek("{"):
            self.pos += 1
            label = self.parse_node_label()
            self.consume("}")
            return (NodeShape.Diamond, label)
        if self.peek("["):
            self.pos += 1
            label = self.parse_node_label()
            self.consume("]")
            return (NodeShape.Rectangle, label)
        return None

    # ── Node ref ──────────────────────────────────────────────────────────────

    def parse_node_ref(self) -> Node | None:
        self.skip_ws()
        node_id = self.match_re(_NODE_ID_RE)
        if not node_id:
            return None
        shape_result = self.parse_node_shape()
        if shape_result:
            shape, label = shape_result
            return Node.new(node_id, label, shape)
        return Node.bare(node_id)

    # ── Edge connector ────────────────────────────────────────────────────────

    def parse_edge_connector(self) -> EdgeType | None:
        self.skip_ws()
        for token, etype in _EDGE_PATTERNS:
            if self.peek(token):
                self.pos += len(token)
                return etype
        return None

    # ── Edge label ────────────────────────────────────────────────────────────

    def try_parse_edge_label(self) -> str | None:
        self.skip_ws()
        if not self.consume("|"):
            return None
        text = self.match_re(_LABEL_TEXT_RE)
        self.consume("|")
        return (text or "").strip()

    # ── Edge chain ────────────────────────────────────────────────────────────

    def parse_edge_chain(self) -> list[tuple[EdgeType, str | None, Node]]:
        segments: list[tuple[EdgeType, str | None, Node]] = []
        while True:
            saved = self.pos
            etype = self.parse_edge_connector()
            if etype is None:
                self.pos = saved
                break
            label = self.try_parse_edge_label()
            node = self.parse_node_ref()
            if node is None:
                self.pos = saved
                break
            segments.append((etype, label, node))
        return segments

    # ── Edge statement ────────────────────────────────────────────────────────

    def try_parse_edge_stmt(self) -> tuple[list[Node], list[Edge]] | None:
        """Try to parse an edge statement. Returns (nodes, edges) or None."""
        saved = self.pos
        source = self.parse_node_ref()
        if source is None:
            self.pos = saved
            return None
        segments = self.parse_edge_chain()
        if not segments:
            self.pos = saved
            return None
        nodes: list[Node] = [source]
        edges: list[Edge] = []
        prev_id = source.id
        for etype, label, target in segments:
            e = Edge.new(prev_id, target.id, etype)
            e.label = label
            prev_id = target.id
            nodes.append(target)
            edges.append(e)
        return (nodes, edges)

    # ── Node statement ────────────────────────────────────────────────────────

    def try_parse_node_stmt(self) -> Node | None:
        saved = self.pos
        node = self.parse_node_ref()
        if node is None:
            self.pos = saved
            return None
        return node

    # ── Subgraph direction ────────────────────────────────────────────────────

    def try_parse_subgraph_direction(self) -> Direction | None:
        saved = self.pos
        self.skip_ws()
        if self.consume("direction"):
            self.skip_ws()
            d = self.parse_direction_value()
            self.skip_ws()
            self.consume_newline()
            return d
        self.pos = saved
        return None

    # ── Subgraph label ────────────────────────────────────────────────────────

    def parse_subgraph_label(self) -> str:
        self.skip_ws()
        if self.pos < len(self.src) and self.src[self.pos] == '"':
            return self.parse_quoted_string()
        label = self.match_re(_BARE_SUBGRAPH_LABEL_RE)
        return (label or "").strip()

    # ── Check for "end" keyword ───────────────────────────────────────────────

    def at_end_keyword(self) -> bool:
        """Return True if we're at the 'end' keyword (not a prefix of longer id)."""
        if not self.src.startswith("end", self.pos):
            return False
        after = self.pos + 3
        if after >= len(self.src):
            return True
        ch = self.src[after]
        return not (ch.isalnum() or ch in ("_", "-"))

    # ── Subgraph block ────────────────────────────────────────────────────────

    def parse_subgraph_block(self) -> Subgraph | None:
        saved = self.pos
        self.skip_ws()
        if not self.consume("subgraph"):
            self.pos = saved
            return None
        # Check that 'subgraph' was a full keyword (not prefix of a node id)
        if self.pos < len(self.src) and (self.src[self.pos].isalnum() or self.src[self.pos] in ("_", "-")):
            self.pos = saved
            return None
        name = self.parse_subgraph_label()
        self.skip_ws()
        self.consume_newline()
        sg = Subgraph.new(name)
        # Optional direction override
        d = self.try_parse_subgraph_direction()
        if d is not None:
            sg.direction = d
        # Statements until "end"
        while not self.eof():
            self.skip_ws()
            if self.at_end_keyword():
                self.pos += 3  # consume 'end'
                self.skip_ws()
                self.consume_newline()
                break
            if not self.parse_statement_into(sg.nodes, sg.edges, sg.subgraphs) and not self.consume_newline():
                self.pos += 1
        return sg

    # ── Statement ─────────────────────────────────────────────────────────────

    def parse_statement_into(
        self,
        nodes: list[Node],
        edges: list[Edge],
        subgraphs: list[Subgraph],
    ) -> bool:
        """Try to parse one statement. Returns True if something was consumed."""
        self.skip_ws()
        if self.eof():
            return False

        # Subgraph
        sg = self.parse_subgraph_block()
        if sg is not None:
            subgraphs.append(sg)
            return True

        # Edge statement (must try before node_stmt since node_stmt is a prefix)
        result = self.try_parse_edge_stmt()
        if result is not None:
            stmt_nodes, stmt_edges = result
            for n in stmt_nodes:
                upsert_node(nodes, n)
            edges.extend(stmt_edges)
            self.skip_ws()
            self.consume_newline()
            return True

        # Node statement
        node = self.try_parse_node_stmt()
        if node is not None:
            upsert_node(nodes, node)
            self.skip_ws()
            self.consume_newline()
            return True

        return False

    # ── Top-level parse ───────────────────────────────────────────────────────

    def parse_graph(self) -> Graph:
        graph = Graph.new()
        direction = self.try_parse_header()
        if direction is not None:
            graph.direction = direction

        while not self.eof():
            self.skip_ws()
            if self.eof():
                break
            # Skip blank lines
            if self.consume_newline():
                continue
            if not self.parse_statement_into(graph.nodes, graph.edges, graph.subgraphs):
                # Skip unrecognized characters to avoid infinite loop
                self.pos += 1

        return graph


# ─── Helpers ─────────────────────────────────────────────────────────────────


def upsert_node(nodes: list[Node], node: Node) -> None:
    """First-definition-wins: insert node only if id not already present."""
    if not any(n.id == node.id for n in nodes):
        nodes.append(node)


# ─── Public API ──────────────────────────────────────────────────────────────


def parse(input: str) -> Graph:
    """Parse Mermaid flowchart text and return a Graph AST.

    Raises ValueError on unrecoverable parse errors.
    """
    p = _Parser(src=input)
    return p.parse_graph()
