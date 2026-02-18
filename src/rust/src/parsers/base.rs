//! Parser trait definition and shared Cursor tokenizer.
//!
//! Mirrors Python's parsers/base.py (Parser protocol) and
//! the _Cursor class from parsers/flowchart.py.

use crate::syntax::types::{Direction, Edge, EdgeType, Graph, Node, NodeShape, Subgraph};

// ─── Parser trait ────────────────────────────────────────────────────────────

/// Trait for diagram parsers.
///
/// Each diagram type (flowchart, sequence, etc.) implements this trait.
pub trait Parser {
    /// Parse the input source string into a Graph AST.
    fn parse(&self, src: &str) -> Result<Graph, String>;
}

// ─── Edge patterns ───────────────────────────────────────────────────────────

/// Edge connector tokens in priority order (longest-match first).
pub const EDGE_PATTERNS: &[(&str, EdgeType)] = &[
    ("<-.->", EdgeType::BidirDotted),
    ("<==>", EdgeType::BidirThick),
    ("<-->", EdgeType::BidirArrow),
    ("-.->", EdgeType::DottedArrow),
    ("==>", EdgeType::ThickArrow),
    ("-->", EdgeType::Arrow),
    ("-.-", EdgeType::DottedLine),
    ("===", EdgeType::ThickLine),
    ("---", EdgeType::Line),
];

// ─── Cursor (stateful tokenizer) ─────────────────────────────────────────────

/// Stateful parser cursor over the input string.
///
/// Mirrors Python's _Cursor dataclass from parsers/flowchart.py.
pub struct Cursor {
    pub src: Vec<char>,
    pub pos: usize,
}

impl Cursor {
    pub fn new(src: &str) -> Self {
        Self {
            src: src.chars().collect(),
            pos: 0,
        }
    }

    pub fn eof(&self) -> bool {
        self.pos >= self.src.len()
    }

    /// Peek whether the next chars match the given ASCII string.
    pub fn peek(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        if self.pos + chars.len() > self.src.len() {
            return false;
        }
        self.src[self.pos..self.pos + chars.len()] == chars[..]
    }

    /// Consume `s` if it matches; returns true if consumed.
    pub fn consume(&mut self, s: &str) -> bool {
        if self.peek(s) {
            self.pos += s.chars().count();
            true
        } else {
            false
        }
    }

    /// Skip horizontal whitespace (spaces/tabs) and `%% ...` comments.
    pub fn skip_ws(&mut self) {
        loop {
            // skip spaces and tabs
            if self.pos < self.src.len()
                && (self.src[self.pos] == ' ' || self.src[self.pos] == '\t')
            {
                self.pos += 1;
                continue;
            }
            // skip %% comment (to end of line)
            if self.peek("%%") {
                self.pos += 2;
                while self.pos < self.src.len() && self.src[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    /// Skip whitespace, comments, and newlines.
    pub fn skip_ws_and_newlines(&mut self) {
        loop {
            if self.pos < self.src.len()
                && (self.src[self.pos] == ' '
                    || self.src[self.pos] == '\t'
                    || self.src[self.pos] == '\r'
                    || self.src[self.pos] == '\n')
            {
                self.pos += 1;
                continue;
            }
            if self.peek("%%") {
                self.pos += 2;
                while self.pos < self.src.len() && self.src[self.pos] != '\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    /// Consume a newline (`\r\n`, `\n`, or `\r`). Returns true if consumed.
    pub fn consume_newline(&mut self) -> bool {
        if self.pos < self.src.len() {
            if self.src[self.pos] == '\r' {
                self.pos += 1;
                if self.pos < self.src.len() && self.src[self.pos] == '\n' {
                    self.pos += 1;
                }
                return true;
            }
            if self.src[self.pos] == '\n' {
                self.pos += 1;
                return true;
            }
        }
        false
    }

    /// Match a node identifier: `[a-zA-Z_][a-zA-Z0-9_-]*`.
    pub fn match_node_id(&mut self) -> Option<String> {
        if self.pos >= self.src.len() {
            return None;
        }
        let ch = self.src[self.pos];
        if !ch.is_ascii_alphabetic() && ch != '_' {
            return None;
        }
        let start = self.pos;
        self.pos += 1;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        Some(self.src[start..self.pos].iter().collect())
    }

    /// Match a direction keyword: `TD`, `TB`, `LR`, `RL`, `BT`.
    pub fn match_direction(&mut self) -> Option<Direction> {
        for (token, dir) in &[
            ("TD", Direction::TD),
            ("TB", Direction::TD),
            ("LR", Direction::LR),
            ("RL", Direction::RL),
            ("BT", Direction::BT),
        ] {
            if self.peek(token) {
                // Make sure it's not followed by alphanumeric (it's a full word)
                let end = self.pos + token.len();
                let followed_by_alnum = end < self.src.len()
                    && (self.src[end].is_ascii_alphanumeric() || self.src[end] == '_');
                if !followed_by_alnum {
                    self.pos += token.len();
                    return Some(dir.clone());
                }
            }
        }
        None
    }

    /// Try to parse the flowchart/graph header. Returns direction if found.
    pub fn try_parse_header(&mut self) -> Option<Direction> {
        let saved = self.pos;
        self.skip_ws_and_newlines();
        let is_flowchart = self.consume("flowchart");
        let is_graph = !is_flowchart && self.consume("graph");
        if is_flowchart || is_graph {
            self.skip_ws();
            let d = self.match_direction().unwrap_or(Direction::TD);
            self.skip_ws();
            // consume optional trailing comment
            if self.peek("%%") {
                self.pos += 2;
                while self.pos < self.src.len() && self.src[self.pos] != '\n' {
                    self.pos += 1;
                }
            }
            self.skip_ws();
            self.consume_newline();
            return Some(d);
        }
        self.pos = saved;
        None
    }

    /// Parse a double-quoted string, handling `\n`, `\"`, `\\` escapes.
    pub fn parse_quoted_string(&mut self) -> String {
        // Caller must have verified src[pos] == '"'
        self.pos += 1;
        let mut buf = String::new();
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == '"' {
                self.pos += 1;
                break;
            }
            if ch == '\\' && self.pos + 1 < self.src.len() {
                let nxt = self.src[self.pos + 1];
                match nxt {
                    'n' => buf.push('\n'),
                    '"' => buf.push('"'),
                    '\\' => buf.push('\\'),
                    other => buf.push(other),
                }
                self.pos += 2;
            } else {
                buf.push(ch);
                self.pos += 1;
            }
        }
        buf
    }

    /// Parse a node label (quoted or bare, up to `]`, `)`, `}`, or newline).
    pub fn parse_node_label(&mut self) -> String {
        self.skip_ws();
        if self.pos < self.src.len() && self.src[self.pos] == '"' {
            return self.parse_quoted_string();
        }
        // Bare label: everything up to ], ), }, newline
        let start = self.pos;
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == ']' || ch == ')' || ch == '}' || ch == '\n' || ch == '\r' {
                break;
            }
            self.pos += 1;
        }
        let label: String = self.src[start..self.pos].iter().collect();
        label.trim().to_string()
    }

    /// Parse a subgraph label (everything to end of line, possibly quoted).
    pub fn parse_subgraph_label(&mut self) -> String {
        self.skip_ws();
        if self.pos < self.src.len() && self.src[self.pos] == '"' {
            return self.parse_quoted_string();
        }
        let start = self.pos;
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.pos += 1;
        }
        let label: String = self.src[start..self.pos].iter().collect();
        label.trim().to_string()
    }

    /// Try to parse a node shape bracket. Returns `(NodeShape, label)` or None.
    pub fn parse_node_shape(&mut self) -> Option<(NodeShape, String)> {
        if self.peek("((") {
            self.pos += 2;
            let label = self.parse_node_label();
            self.consume("))");
            return Some((NodeShape::Circle, label));
        }
        if self.pos < self.src.len() && self.src[self.pos] == '(' && !self.peek("((") {
            self.pos += 1;
            let label = self.parse_node_label();
            self.consume(")");
            return Some((NodeShape::Rounded, label));
        }
        if self.pos < self.src.len() && self.src[self.pos] == '{' {
            self.pos += 1;
            let label = self.parse_node_label();
            self.consume("}");
            return Some((NodeShape::Diamond, label));
        }
        if self.pos < self.src.len() && self.src[self.pos] == '[' {
            self.pos += 1;
            let label = self.parse_node_label();
            self.consume("]");
            return Some((NodeShape::Rectangle, label));
        }
        None
    }

    /// Parse a node reference (id + optional shape bracket).
    pub fn parse_node_ref(&mut self) -> Option<Node> {
        self.skip_ws();
        let node_id = self.match_node_id()?;
        if let Some((shape, label)) = self.parse_node_shape() {
            Some(Node::new(node_id, label, shape))
        } else {
            Some(Node::bare(node_id))
        }
    }

    /// Try to parse an edge connector token. Returns EdgeType or None.
    pub fn parse_edge_connector(&mut self) -> Option<EdgeType> {
        self.skip_ws();
        for (token, etype) in EDGE_PATTERNS {
            if self.peek(token) {
                self.pos += token.chars().count();
                return Some(etype.clone());
            }
        }
        None
    }

    /// Try to parse an edge label `|text|`. Returns label text or None.
    pub fn try_parse_edge_label(&mut self) -> Option<String> {
        self.skip_ws();
        if !self.consume("|") {
            return None;
        }
        let start = self.pos;
        while self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch == '|' || ch == '\n' || ch == '\r' {
                break;
            }
            self.pos += 1;
        }
        let text: String = self.src[start..self.pos].iter().collect();
        self.consume("|");
        Some(text.trim().to_string())
    }

    /// Parse an edge chain: `connector [label] target [connector [label] target ...]`.
    pub fn parse_edge_chain(&mut self) -> Vec<(EdgeType, Option<String>, Node)> {
        let mut segments = Vec::new();
        loop {
            let saved = self.pos;
            let Some(etype) = self.parse_edge_connector() else {
                self.pos = saved;
                break;
            };
            let label = self.try_parse_edge_label();
            let Some(node) = self.parse_node_ref() else {
                self.pos = saved;
                break;
            };
            segments.push((etype, label, node));
        }
        segments
    }

    /// Try to parse an edge statement. Returns (nodes, edges) or None.
    pub fn try_parse_edge_stmt(&mut self) -> Option<(Vec<Node>, Vec<Edge>)> {
        let saved = self.pos;
        let source = self.parse_node_ref()?;
        let segments = self.parse_edge_chain();
        if segments.is_empty() {
            self.pos = saved;
            return None;
        }
        let mut nodes: Vec<Node> = vec![source.clone()];
        let mut edges: Vec<Edge> = Vec::new();
        let mut prev_id = source.id.clone();
        for (etype, label, target) in segments {
            let mut e = Edge::new(prev_id.clone(), target.id.clone(), etype);
            e.label = label;
            prev_id = target.id.clone();
            nodes.push(target);
            edges.push(e);
        }
        Some((nodes, edges))
    }

    /// Try to parse a standalone node statement. Returns Node or None.
    pub fn try_parse_node_stmt(&mut self) -> Option<Node> {
        let saved = self.pos;
        let node = self.parse_node_ref()?;
        // A bare identifier that is also a keyword should not be treated as a node.
        // (The keyword checks happen before this call, so we just return the node.)
        let _ = saved;
        Some(node)
    }

    /// Try to parse a `direction` directive inside a subgraph.
    pub fn try_parse_subgraph_direction(&mut self) -> Option<Direction> {
        let saved = self.pos;
        self.skip_ws();
        if self.consume("direction") {
            self.skip_ws();
            if let Some(d) = self.match_direction() {
                self.skip_ws();
                self.consume_newline();
                return Some(d);
            }
        }
        self.pos = saved;
        None
    }

    /// Check if cursor is at the `end` keyword (word boundary).
    pub fn at_end_keyword(&self) -> bool {
        if self.pos + 3 > self.src.len() {
            return false;
        }
        if self.src[self.pos] != 'e'
            || self.src[self.pos + 1] != 'n'
            || self.src[self.pos + 2] != 'd'
        {
            return false;
        }
        let after = self.pos + 3;
        if after >= self.src.len() {
            return true;
        }
        let ch = self.src[after];
        !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    }

    /// Parse a `subgraph ... end` block. Returns Subgraph or None.
    pub fn parse_subgraph_block(&mut self) -> Option<Subgraph> {
        let saved = self.pos;
        self.skip_ws();
        if !self.consume("subgraph") {
            self.pos = saved;
            return None;
        }
        // Must not be followed by identifier char (e.g. "subgraphFoo" is a node id)
        if self.pos < self.src.len() {
            let ch = self.src[self.pos];
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                self.pos = saved;
                return None;
            }
        }
        let name = self.parse_subgraph_label();
        self.skip_ws();
        self.consume_newline();
        let mut sg = Subgraph::new(name);
        if let Some(d) = self.try_parse_subgraph_direction() {
            sg.direction = Some(d);
        }
        while !self.eof() {
            self.skip_ws();
            if self.at_end_keyword() {
                self.pos += 3;
                self.skip_ws();
                self.consume_newline();
                break;
            }
            if !self.parse_statement_into(&mut sg.nodes, &mut sg.edges, &mut sg.subgraphs)
                && !self.consume_newline()
            {
                self.pos += 1;
            }
        }
        Some(sg)
    }

    /// Parse one statement into the given node/edge/subgraph lists.
    /// Returns true if a statement was consumed.
    pub fn parse_statement_into(
        &mut self,
        nodes: &mut Vec<Node>,
        edges: &mut Vec<Edge>,
        subgraphs: &mut Vec<Subgraph>,
    ) -> bool {
        self.skip_ws();
        if self.eof() {
            return false;
        }

        if let Some(sg) = self.parse_subgraph_block() {
            subgraphs.push(sg);
            return true;
        }

        if let Some((stmt_nodes, stmt_edges)) = self.try_parse_edge_stmt() {
            for n in stmt_nodes {
                upsert_node(nodes, n);
            }
            edges.extend(stmt_edges);
            self.skip_ws();
            self.consume_newline();
            return true;
        }

        if let Some(node) = self.try_parse_node_stmt() {
            upsert_node(nodes, node);
            self.skip_ws();
            self.consume_newline();
            return true;
        }

        false
    }

    /// Parse the full graph (header + statements).
    pub fn parse_graph(&mut self) -> Graph {
        let mut graph = Graph::new();
        if let Some(direction) = self.try_parse_header() {
            graph.direction = direction;
        }
        while !self.eof() {
            self.skip_ws();
            if self.eof() {
                break;
            }
            if self.consume_newline() {
                continue;
            }
            if !self.parse_statement_into(&mut graph.nodes, &mut graph.edges, &mut graph.subgraphs)
            {
                self.pos += 1;
            }
        }
        graph
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// First-definition-wins: insert node only if id not already present.
pub fn upsert_node(nodes: &mut Vec<Node>, node: Node) {
    if !nodes.iter().any(|n| n.id == node.id) {
        nodes.push(node);
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_eof() {
        let c = Cursor::new("");
        assert!(c.eof());
        let c = Cursor::new("a");
        assert!(!c.eof());
    }

    #[test]
    fn test_cursor_peek_consume() {
        let mut c = Cursor::new("-->rest");
        assert!(c.peek("-->"));
        assert!(!c.peek("-.-"));
        assert!(c.consume("-->"));
        assert_eq!(c.pos, 3);
    }

    #[test]
    fn test_skip_ws() {
        let mut c = Cursor::new("  \t  foo");
        c.skip_ws();
        assert_eq!(c.pos, 5);
    }

    #[test]
    fn test_skip_ws_comment() {
        let mut c = Cursor::new("%% comment\nfoo");
        c.skip_ws();
        // should consume %% comment but stop before \n
        assert_eq!(&c.src[c.pos..].iter().collect::<String>(), "\nfoo");
    }

    #[test]
    fn test_match_node_id() {
        let mut c = Cursor::new("my-node rest");
        let id = c.match_node_id().unwrap();
        assert_eq!(id, "my-node");
    }

    #[test]
    fn test_match_direction() {
        let mut c = Cursor::new("TD");
        assert_eq!(c.match_direction(), Some(Direction::TD));
        let mut c = Cursor::new("LR");
        assert_eq!(c.match_direction(), Some(Direction::LR));
    }

    #[test]
    fn test_parse_quoted_string() {
        let mut c = Cursor::new("\"Hello\\nWorld\"");
        let s = c.parse_quoted_string();
        assert_eq!(s, "Hello\nWorld");
    }

    #[test]
    fn test_parse_node_shape_rect() {
        let mut c = Cursor::new("[Start]");
        let (shape, label) = c.parse_node_shape().unwrap();
        assert_eq!(shape, NodeShape::Rectangle);
        assert_eq!(label, "Start");
    }

    #[test]
    fn test_parse_node_shape_circle() {
        let mut c = Cursor::new("((DB))");
        let (shape, label) = c.parse_node_shape().unwrap();
        assert_eq!(shape, NodeShape::Circle);
        assert_eq!(label, "DB");
    }

    #[test]
    fn test_parse_node_ref_bare() {
        let mut c = Cursor::new("A");
        let node = c.parse_node_ref().unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label, "A");
        assert_eq!(node.shape, NodeShape::Rectangle);
    }

    #[test]
    fn test_parse_node_ref_with_label() {
        let mut c = Cursor::new("A[Hello]");
        let node = c.parse_node_ref().unwrap();
        assert_eq!(node.id, "A");
        assert_eq!(node.label, "Hello");
        assert_eq!(node.shape, NodeShape::Rectangle);
    }

    #[test]
    fn test_parse_edge_connector() {
        let mut c = Cursor::new("-->");
        assert_eq!(c.parse_edge_connector(), Some(EdgeType::Arrow));
        let mut c = Cursor::new("-.-");
        assert_eq!(c.parse_edge_connector(), Some(EdgeType::DottedLine));
    }

    #[test]
    fn test_try_parse_edge_label() {
        let mut c = Cursor::new("|yes|");
        assert_eq!(c.try_parse_edge_label(), Some("yes".to_string()));
    }

    #[test]
    fn test_upsert_node_first_wins() {
        let mut nodes = vec![Node::new("A", "First", NodeShape::Rectangle)];
        upsert_node(&mut nodes, Node::new("A", "Second", NodeShape::Rounded));
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].label, "First");
    }

    #[test]
    fn test_at_end_keyword() {
        let c = Cursor::new("end");
        assert!(c.at_end_keyword());
        let c = Cursor::new("endgame");
        assert!(!c.at_end_keyword());
    }

    #[test]
    fn test_try_parse_header() {
        let mut c = Cursor::new("graph TD\n");
        let dir = c.try_parse_header().unwrap();
        assert_eq!(dir, Direction::TD);

        let mut c = Cursor::new("flowchart LR\n");
        let dir = c.try_parse_header().unwrap();
        assert_eq!(dir, Direction::LR);
    }

    #[test]
    fn test_parse_graph_simple() {
        let mut c = Cursor::new("graph TD\n    A --> B\n");
        let g = c.parse_graph();
        assert_eq!(g.direction, Direction::TD);
        assert_eq!(g.nodes.len(), 2);
        assert_eq!(g.edges.len(), 1);
        assert_eq!(g.edges[0].from_id, "A");
        assert_eq!(g.edges[0].to_id, "B");
    }

    #[test]
    fn test_parse_graph_no_header() {
        let mut c = Cursor::new("A --> B\n");
        let g = c.parse_graph();
        assert_eq!(g.direction, Direction::TD); // default
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn test_parse_graph_edge_label() {
        let mut c = Cursor::new("graph TD\n    A -->|yes| B\n");
        let g = c.parse_graph();
        assert_eq!(g.edges[0].label, Some("yes".to_string()));
    }

    #[test]
    fn test_parse_graph_subgraph() {
        let mut c = Cursor::new("graph TD\n    subgraph Group\n        A --> B\n    end\n");
        let g = c.parse_graph();
        assert_eq!(g.subgraphs.len(), 1);
        assert_eq!(g.subgraphs[0].name, "Group");
        assert_eq!(g.subgraphs[0].nodes.len(), 2);
    }

    #[test]
    fn test_parse_graph_all_shapes() {
        let mut c =
            Cursor::new("graph TD\n    A[Rect] --> B(Round) --> C{Diamond} --> D((Circle))\n");
        let g = c.parse_graph();
        assert_eq!(g.nodes[0].shape, NodeShape::Rectangle);
        assert_eq!(g.nodes[1].shape, NodeShape::Rounded);
        assert_eq!(g.nodes[2].shape, NodeShape::Diamond);
        assert_eq!(g.nodes[3].shape, NodeShape::Circle);
    }

    #[test]
    fn test_parse_graph_comment() {
        let mut c = Cursor::new("graph TD\n    %% This is a comment\n    A --> B\n");
        let g = c.parse_graph();
        assert_eq!(g.nodes.len(), 2);
    }

    #[test]
    fn test_parse_edge_types() {
        let mut c = Cursor::new("graph TD\n    A --> B\n    C --- D\n    E -.-> F\n    G ==> H\n");
        let g = c.parse_graph();
        assert_eq!(g.edges[0].edge_type, EdgeType::Arrow);
        assert_eq!(g.edges[1].edge_type, EdgeType::Line);
        assert_eq!(g.edges[2].edge_type, EdgeType::DottedArrow);
        assert_eq!(g.edges[3].edge_type, EdgeType::ThickArrow);
    }
}
