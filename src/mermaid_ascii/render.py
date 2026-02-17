"""Render module — Phase 5 of the pipeline.

Paints layout data (positioned nodes + routed edges) onto a 2D character
grid and converts it to a printable string.

Layer order (paint last wins):
  1. Subgraph borders
  2. Node boxes
  3. Edge lines (horizontal and vertical segments)
  4. Edge corners / junctions (merged using Unicode box-drawing rules)
  5. Arrowheads
  6. Edge labels

Character sets:
  - Unicode box-drawing (default): ┌ ┐ └ ┘ ─ │ ├ ┤ ┬ ┴ ┼ ► ▼ ◄ ▲
  - ASCII fallback:                + + + + - | + + + + + > v < ^

1:1 port of render.rs.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

from mermaid_ascii.ast import Direction, EdgeType, NodeShape
from mermaid_ascii.graph import GraphIR
from mermaid_ascii.layout import COMPOUND_PREFIX, DUMMY_PREFIX, LayoutNode, RoutedEdge

# ─── Geometry Types ───────────────────────────────────────────────────────────


@dataclass
class Rect:
    """A rectangle in character coordinates (top-left origin)."""

    x: int
    y: int
    width: int
    height: int

    def right(self) -> int:
        """Right edge column (exclusive)."""
        return self.x + self.width

    def bottom(self) -> int:
        """Bottom row (exclusive)."""
        return self.y + self.height


# ─── Character Set ────────────────────────────────────────────────────────────


class CharSet(Enum):
    """Which character set to use for box-drawing."""

    Unicode = "unicode"
    Ascii = "ascii"


@dataclass
class BoxChars:
    """All the characters needed to draw boxes and edges."""

    top_left: str
    top_right: str
    bottom_left: str
    bottom_right: str
    horizontal: str
    vertical: str
    tee_right: str
    tee_left: str
    tee_down: str
    tee_up: str
    cross: str
    arrow_right: str
    arrow_left: str
    arrow_down: str
    arrow_up: str

    @classmethod
    def unicode(cls) -> BoxChars:
        return cls(
            top_left="┌",
            top_right="┐",
            bottom_left="└",
            bottom_right="┘",
            horizontal="─",
            vertical="│",
            tee_right="├",
            tee_left="┤",
            tee_down="┬",
            tee_up="┴",
            cross="┼",
            arrow_right="►",
            arrow_left="◄",
            arrow_down="▼",
            arrow_up="▲",
        )

    @classmethod
    def ascii(cls) -> BoxChars:
        return cls(
            top_left="+",
            top_right="+",
            bottom_left="+",
            bottom_right="+",
            horizontal="-",
            vertical="|",
            tee_right="+",
            tee_left="+",
            tee_down="+",
            tee_up="+",
            cross="+",
            arrow_right=">",
            arrow_left="<",
            arrow_down="v",
            arrow_up="^",
        )

    @classmethod
    def for_charset(cls, cs: CharSet) -> BoxChars:
        if cs == CharSet.Unicode:
            return cls.unicode()
        return cls.ascii()


# ─── Junction Merging ─────────────────────────────────────────────────────────


@dataclass
class Arms:
    """Describes which of the four arms of a cell are active (connected).

    Used when deciding what junction character to place at a cell where two
    edges meet. For example, if a vertical edge crosses a horizontal edge we
    get {up, down, left, right} → ┼.
    """

    up: bool = False
    down: bool = False
    left: bool = False
    right: bool = False

    @classmethod
    def from_char(cls, c: str) -> Arms | None:
        """Compute the Arms implied by an existing box-drawing character.

        Returns None if the character is not a recognised box-drawing char.
        """
        table: dict[str, tuple[bool, bool, bool, bool]] = {
            "─": (False, False, True, True),
            "│": (True, True, False, False),
            "┌": (False, True, False, True),
            "┐": (False, True, True, False),
            "└": (True, False, False, True),
            "┘": (True, False, True, False),
            "├": (True, True, False, True),
            "┤": (True, True, True, False),
            "┬": (False, True, True, True),
            "┴": (True, False, True, True),
            "┼": (True, True, True, True),
            # ASCII equivalents — map to same topology.
            "-": (False, False, True, True),
            "|": (True, True, False, False),
            "+": (True, True, True, True),
        }
        entry = table.get(c)
        if entry is None:
            return None
        u, d, lft, r = entry
        return cls(up=u, down=d, left=lft, right=r)

    def merge(self, other: Arms) -> Arms:
        """Merge two Arms by OR-ing their bits."""
        return Arms(
            up=self.up or other.up,
            down=self.down or other.down,
            left=self.left or other.left,
            right=self.right or other.right,
        )

    def to_char(self, cs: CharSet) -> str:
        """Convert the combined arms back to a Unicode box-drawing character."""
        bc = BoxChars.for_charset(cs)
        key = (self.up, self.down, self.left, self.right)
        match key:
            case (False, False, False, False):
                return " "
            # Straight lines.
            case (False, False, True, True):
                return bc.horizontal
            case (True, True, False, False):
                return bc.vertical
            # Corners.
            case (False, True, False, True):
                return bc.top_left
            case (False, True, True, False):
                return bc.top_right
            case (True, False, False, True):
                return bc.bottom_left
            case (True, False, True, False):
                return bc.bottom_right
            # Tees.
            case (True, True, False, True):
                return bc.tee_right
            case (True, True, True, False):
                return bc.tee_left
            case (False, True, True, True):
                return bc.tee_down
            case (True, False, True, True):
                return bc.tee_up
            # Full cross.
            case (True, True, True, True):
                return bc.cross
            # Partial / single arm — treat as the nearest line or a corner.
            case (True, False, False, False):
                return bc.vertical
            case (False, True, False, False):
                return bc.vertical
            case (False, False, True, False):
                return bc.horizontal
            case (False, False, False, True):
                return bc.horizontal
            case _:
                return " "


# ─── Canvas ───────────────────────────────────────────────────────────────────


class Canvas:
    """A 2D character grid onto which graph elements are painted.

    The canvas uses a row-major layout: cells[row][col].
    All coordinates are in character units (column = x, row = y).
    """

    def __init__(self, width: int, height: int, charset: CharSet) -> None:
        self.width = width
        self.height = height
        self.charset = charset
        self.cells: list[list[str]] = [[" "] * width for _ in range(height)]

    def get(self, col: int, row: int) -> str:
        """Read the character at (col, row). Returns ' ' if out of bounds."""
        if 0 <= row < self.height and 0 <= col < self.width:
            return self.cells[row][col]
        return " "

    def set(self, col: int, row: int, c: str) -> None:
        """Write a character at (col, row), ignoring out-of-bounds writes."""
        if 0 <= row < self.height and 0 <= col < self.width:
            self.cells[row][col] = c

    def set_merge(self, col: int, row: int, c: str) -> None:
        """Write a character at (col, row) using junction merging.

        If the current cell already contains a recognised box-drawing character,
        the new character is merged with it so that all active arms are preserved.
        For example, painting ─ over │ yields ┼.

        Falls back to simple overwrite when either character is not a
        box-drawing character (e.g. writing a letter over a space).
        """
        if row >= self.height or col >= self.width:
            return
        existing = self.cells[row][col]
        ea = Arms.from_char(existing)
        na = Arms.from_char(c)
        if ea is not None and na is not None:
            merged = ea.merge(na)
            self.cells[row][col] = merged.to_char(self.charset)
        else:
            self.cells[row][col] = c

    # ─── Primitive drawing operations ────────────────────────────────────────

    def hline(self, y: int, x1: int, x2: int, c: str) -> None:
        """Draw a horizontal line of c from column x1 to x2 (inclusive) at row y."""
        lo, hi = (x1, x2) if x1 <= x2 else (x2, x1)
        for col in range(lo, hi + 1):
            self.set_merge(col, y, c)

    def vline(self, x: int, y1: int, y2: int, c: str) -> None:
        """Draw a vertical line of c from row y1 to y2 (inclusive) at column x."""
        lo, hi = (y1, y2) if y1 <= y2 else (y2, y1)
        for row in range(lo, hi + 1):
            self.set_merge(x, row, c)

    def draw_box(self, rect: Rect, bc: BoxChars) -> None:
        """Draw a box outline described by rect, using bc box characters."""
        if rect.width < 2 or rect.height < 2:
            return
        x0 = rect.x
        y0 = rect.y
        x1 = rect.x + rect.width - 1  # right column
        y1 = rect.y + rect.height - 1  # bottom row

        # Corners.
        self.set(x0, y0, bc.top_left)
        self.set(x1, y0, bc.top_right)
        self.set(x0, y1, bc.bottom_left)
        self.set(x1, y1, bc.bottom_right)

        # Top and bottom horizontal edges (inside the corners).
        for col in range(x0 + 1, x1):
            self.set(col, y0, bc.horizontal)
            self.set(col, y1, bc.horizontal)

        # Left and right vertical edges (inside the corners).
        for row in range(y0 + 1, y1):
            self.set(x0, row, bc.vertical)
            self.set(x1, row, bc.vertical)

    def write_str(self, col: int, row: int, s: str) -> None:
        """Write a string starting at (col, row). Clips at canvas boundary."""
        for i, ch in enumerate(s):
            c = col + i
            if c >= self.width or row >= self.height:
                break
            self.cells[row][c] = ch

    # ─── Render to string ─────────────────────────────────────────────────────

    def to_string(self) -> str:
        """Convert the canvas to a printable string.

        Each row becomes one line. Trailing spaces on each line are stripped.
        """
        lines = []
        for row in self.cells:
            line = "".join(row).rstrip()
            lines.append(line)
        out = "\n".join(lines)
        trimmed = out.rstrip("\n")
        return trimmed + "\n"


# ─── Node Rendering ───────────────────────────────────────────────────────────


def box_chars_for_shape(shape: NodeShape, cs: CharSet) -> BoxChars:
    """Box characters for each node shape."""
    if shape == NodeShape.Rectangle:
        return BoxChars.for_charset(cs)
    elif shape == NodeShape.Rounded:
        if cs == CharSet.Ascii:
            return BoxChars.ascii()
        bc = BoxChars.unicode()
        bc.top_left = "╭"
        bc.top_right = "╮"
        bc.bottom_left = "╰"
        bc.bottom_right = "╯"
        return bc
    elif shape == NodeShape.Diamond:
        bc = BoxChars.for_charset(cs)
        bc.top_left = "/"
        bc.top_right = "\\"
        bc.bottom_left = "\\"
        bc.bottom_right = "/"
        return bc
    else:  # Circle
        bc = BoxChars.for_charset(cs)
        bc.top_left = "("
        bc.top_right = ")"
        bc.bottom_left = "("
        bc.bottom_right = ")"
        bc.vertical = " "
        return bc


def paint_node(canvas: Canvas, ln: LayoutNode, shape: NodeShape, label: str) -> None:
    """Paint a single node box with its label onto the canvas."""
    x = ln.x
    y = ln.y
    w = ln.width
    h = ln.height

    bc = box_chars_for_shape(shape, canvas.charset)

    rect = Rect(x, y, w, h)
    canvas.draw_box(rect, bc)

    inner_w = max(0, w - 2)
    lines = label.split("\n")

    for i, line in enumerate(lines):
        label_row = y + 1 + i
        line_len = len(line)
        pad = max(0, inner_w - line_len) // 2
        col_start = x + 1 + pad
        canvas.write_str(col_start, label_row, line)


# ─── Compound Node / Subgraph Border Rendering ───────────────────────────────


def paint_compound_node(canvas: Canvas, ln: LayoutNode, sg_name: str, description: str | None) -> None:
    """Paint a compound node as a subgraph border box with title inside."""
    bc = BoxChars.for_charset(canvas.charset)

    rect = Rect(ln.x, ln.y, ln.width, ln.height)
    canvas.draw_box(rect, bc)

    inner_w = max(0, ln.width - 2)
    title_pad = max(0, inner_w - len(sg_name)) // 2
    title_col = ln.x + 1 + title_pad
    title_row = ln.y + 1
    canvas.write_str(title_col, title_row, sg_name)

    if description is not None:
        desc_row = ln.y + ln.height - 2
        desc_pad = max(0, inner_w - len(description)) // 2
        desc_col = ln.x + 1 + desc_pad
        canvas.write_str(desc_col, desc_row, description)


def paint_subgraph_borders(gir: GraphIR, layout_nodes: list[LayoutNode], canvas: Canvas) -> None:
    """Paint subgraph borders for non-compound subgraphs (legacy fallback)."""
    node_pos: dict[str, LayoutNode] = {n.id: n for n in layout_nodes}

    bc = BoxChars.for_charset(canvas.charset)

    for sg_name, members in gir.subgraph_members.items():
        if not members:
            continue

        min_x = min_y = 10**9
        max_x = max_y = 0

        for member_id in members:
            ln = node_pos.get(member_id)
            if ln is None:
                continue
            if ln.x < min_x:
                min_x = ln.x
            if ln.y < min_y:
                min_y = ln.y
            right = ln.x + ln.width
            bottom = ln.y + ln.height
            if right > max_x:
                max_x = right
            if bottom > max_y:
                max_y = bottom

        if min_x == 10**9:
            continue  # no positioned members

        margin_x = 2
        margin_y = 1
        bx = max(0, min_x - margin_x)
        by = max(0, min_y - margin_y)
        bw = (max_x + margin_x) - bx
        bh = (max_y + margin_y) - by

        rect = Rect(bx, by, bw, bh)
        canvas.draw_box(rect, bc)

        label = f" {sg_name} "
        label_col = bx + 2
        if len(label) + 4 <= bw:
            canvas.write_str(label_col, by, label)


# ─── Edge Rendering ───────────────────────────────────────────────────────────


def line_chars_for(edge_type: EdgeType, cs: CharSet) -> tuple[str, str]:
    """Select horizontal and vertical line characters for an edge type."""
    bc = BoxChars.for_charset(cs)
    if edge_type in (EdgeType.ThickArrow, EdgeType.ThickLine, EdgeType.BidirThick):
        return ("═", "║")
    elif edge_type in (EdgeType.DottedArrow, EdgeType.DottedLine, EdgeType.BidirDotted):
        return ("╌", "╎")
    else:
        return (bc.horizontal, bc.vertical)


def paint_edge(canvas: Canvas, re: RoutedEdge, edge_type: EdgeType) -> None:
    """Paint a single routed edge: line segments + arrowhead + optional label."""
    if len(re.waypoints) < 2:
        return

    cs = canvas.charset
    h_ch, v_ch = line_chars_for(edge_type, cs)
    bc = BoxChars.for_charset(cs)

    # Draw each segment between consecutive waypoints.
    for i in range(len(re.waypoints) - 1):
        p0 = re.waypoints[i]
        p1 = re.waypoints[i + 1]

        if p0.y == p1.y:
            canvas.hline(p0.y, p0.x, p1.x, h_ch)
        elif p0.x == p1.x:
            canvas.vline(p0.x, p0.y, p1.y, v_ch)
        # Diagonal segments not supported (orthogonal routing only).

    # Arrowhead placement depends on edge type.
    arrow_types = {
        EdgeType.Arrow,
        EdgeType.DottedArrow,
        EdgeType.ThickArrow,
        EdgeType.BidirArrow,
        EdgeType.BidirDotted,
        EdgeType.BidirThick,
    }
    bidir_types = {EdgeType.BidirArrow, EdgeType.BidirDotted, EdgeType.BidirThick}

    arrow_at_end = edge_type in arrow_types
    arrow_at_start = edge_type in bidir_types

    if arrow_at_end:
        last = re.waypoints[-1]
        prev = re.waypoints[-2]
        if last.y < prev.y:
            arrow = bc.arrow_up
        elif last.y > prev.y:
            arrow = bc.arrow_down
        elif last.x > prev.x:
            arrow = bc.arrow_right
        else:
            arrow = bc.arrow_left
        canvas.set(last.x, last.y, arrow)

    if arrow_at_start:
        first = re.waypoints[0]
        second = re.waypoints[1]
        if first.y < second.y:
            start_arrow = bc.arrow_up
        elif first.y > second.y:
            start_arrow = bc.arrow_down
        elif first.x > second.x:
            start_arrow = bc.arrow_right
        else:
            start_arrow = bc.arrow_left
        canvas.set(first.x, first.y, start_arrow)

    # Edge label: placed at the midpoint waypoint, one row above the line.
    if re.label is not None:
        mid = len(re.waypoints) // 2
        lp = re.waypoints[mid]
        label_y = max(0, lp.y - 1)
        canvas.write_str(lp.x, label_y, re.label)


# ─── Direction Transform Helpers ──────────────────────────────────────────────


def transpose_layout(nodes: list[LayoutNode], edges: list[RoutedEdge]) -> None:
    """Transpose layout coordinates: swap x↔y and width↔height for all nodes
    and edge waypoints.

    Used for LR/RL directions.
    """
    for n in nodes:
        n.x, n.y = n.y, n.x
        n.width, n.height = n.height, n.width
    for re in edges:
        for p in re.waypoints:
            p.x, p.y = p.y, p.x


def remap_char_vertical(c: str) -> str:
    """Remap a single character for vertical flip (BT direction)."""
    table = {
        "▼": "▲",
        "▲": "▼",
        "v": "^",
        "^": "v",
        "┌": "└",
        "└": "┌",
        "┐": "┘",
        "┘": "┐",
        "╭": "╰",
        "╰": "╭",
        "╮": "╯",
        "╯": "╮",
        "┬": "┴",
        "┴": "┬",
    }
    return table.get(c, c)


def remap_char_horizontal(c: str) -> str:
    """Remap a single character for horizontal flip (RL direction)."""
    table = {
        "►": "◄",
        "◄": "►",
        ">": "<",
        "<": ">",
        "┌": "┐",
        "┐": "┌",
        "└": "┘",
        "┘": "└",
        "╭": "╮",
        "╮": "╭",
        "╰": "╯",
        "╯": "╰",
        "├": "┤",
        "┤": "├",
    }
    return table.get(c, c)


def flip_vertical(s: str) -> str:
    """Flip a rendered graph string vertically for BT direction."""
    lines = s.splitlines()
    flipped = ["".join(remap_char_vertical(c) for c in line) for line in reversed(lines)]
    return "\n".join(flipped) + "\n"


def flip_horizontal(s: str) -> str:
    """Flip a rendered graph string horizontally for RL direction."""
    lines = s.splitlines()
    max_width = max((len(line) for line in lines), default=0)
    flipped = []
    for line in lines:
        chars = list(line)
        pad = max_width - len(chars)
        chars.extend([" "] * pad)
        chars.reverse()
        remapped = "".join(remap_char_horizontal(c) for c in chars)
        flipped.append(remapped.rstrip())
    return "\n".join(flipped) + "\n"


# ─── Public Render Entry Point ────────────────────────────────────────────────


def canvas_dimensions(layout_nodes: list[LayoutNode], routed_edges: list[RoutedEdge]) -> tuple[int, int]:
    """Compute the canvas dimensions needed to fit all nodes and edge waypoints."""
    max_col = 40
    max_row = 10

    for n in layout_nodes:
        if n.id.startswith(DUMMY_PREFIX):
            continue
        max_col = max(max_col, n.x + n.width + 2)
        max_row = max(max_row, n.y + n.height + 4)

    for re in routed_edges:
        for p in re.waypoints:
            max_col = max(max_col, p.x + 4)
            max_row = max(max_row, p.y + 4)

    return (max_col, max_row)


def render(
    gir: GraphIR,
    layout_nodes: list[LayoutNode],
    routed_edges: list[RoutedEdge],
    unicode: bool = True,
) -> str:
    """Render a fully-laid-out graph to a multi-line String.

    Args:
        gir: The graph IR (provides node shapes, subgraph membership, edge types, direction).
        layout_nodes: Positioned nodes from the layout phase (may include dummy nodes).
        routed_edges: Routed edges with waypoints from the edge routing phase.
        unicode: True for Unicode box-drawing; False for ASCII fallback.

    Direction transforms applied:
    - LR/RL: transpose_layout swaps x↔y (layout ran with swapped dimensions).
    - BT: flip_vertical reverses rows and remaps directional characters.
    - RL: flip_horizontal (after transpose) reverses columns and remaps chars.
    """
    cs = CharSet.Unicode if unicode else CharSet.Ascii

    # Apply direction-specific coordinate transforms before painting.
    if gir.direction in (Direction.TD, Direction.BT):
        nodes = list(layout_nodes)
        edges = list(routed_edges)
    else:  # LR or RL
        nodes = [
            LayoutNode(
                id=n.id,
                layer=n.layer,
                order=n.order,
                x=n.x,
                y=n.y,
                width=n.width,
                height=n.height,
            )
            for n in layout_nodes
        ]
        from mermaid_ascii.layout import Point

        edges = [
            RoutedEdge(
                from_id=re.from_id,
                to_id=re.to_id,
                label=re.label,
                edge_type=re.edge_type,
                waypoints=[Point(x=p.x, y=p.y) for p in re.waypoints],
            )
            for re in routed_edges
        ]
        transpose_layout(nodes, edges)

    # Separate nodes into categories.
    has_compounds = any(n.id.startswith(COMPOUND_PREFIX) for n in nodes)

    real_nodes = [n for n in nodes if not n.id.startswith(DUMMY_PREFIX) and not n.id.startswith(COMPOUND_PREFIX)]
    compound_nodes = [n for n in nodes if n.id.startswith(COMPOUND_PREFIX)]

    if not real_nodes and not compound_nodes:
        return ""

    width, height = canvas_dimensions(nodes, edges)
    canvas = Canvas(width, height, cs)

    # Build id → NodeData for shape / label lookup.
    node_data_map = {}
    for node_id in gir.digraph.nodes:
        node_data = gir.digraph.nodes[node_id].get("data")
        if node_data is not None:
            node_data_map[node_id] = node_data

    # 1. Subgraph borders.
    if has_compounds:
        for ln in compound_nodes:
            sg_name = ln.id[len(COMPOUND_PREFIX) :]
            desc = gir.subgraph_descriptions.get(sg_name)
            paint_compound_node(canvas, ln, sg_name, desc)
    else:
        paint_subgraph_borders(gir, nodes, canvas)

    # 2. Node boxes + labels.
    for ln in real_nodes:
        nd = node_data_map.get(ln.id)
        shape = nd.shape if nd is not None else NodeShape.Rectangle
        label = nd.label if nd is not None else ln.id
        paint_node(canvas, ln, shape, label)

    # 3–5. Edges: line segments, arrowheads, labels.
    for re in edges:
        paint_edge(canvas, re, re.edge_type)

    rendered = canvas.to_string()

    # Apply post-render flips for directions that need them.
    if gir.direction == Direction.BT:
        return flip_vertical(rendered)
    elif gir.direction == Direction.RL:
        return flip_horizontal(rendered)
    else:
        return rendered
