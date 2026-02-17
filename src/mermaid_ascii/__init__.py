"""mermaid-ascii: Mermaid flowchart syntax to ASCII/Unicode text output."""

from mermaid_ascii.ast import Direction
from mermaid_ascii.graph import GraphIR
from mermaid_ascii.layout import full_layout_with_padding
from mermaid_ascii.parser import parse
from mermaid_ascii.render import render

_DIRECTION_MAP: dict[str, Direction] = {
    "LR": Direction.LR,
    "RL": Direction.RL,
    "TD": Direction.TD,
    "TB": Direction.TD,
    "BT": Direction.BT,
}


def _apply_direction(ast_graph, direction: str | None) -> None:
    """Override ast_graph.direction from a string, if provided."""
    if direction is None:
        return
    key = direction.upper()
    if key not in _DIRECTION_MAP:
        raise ValueError(f"Unknown direction '{direction}'; use LR, RL, TD, or BT")
    ast_graph.direction = _DIRECTION_MAP[key]


def render_dsl(src: str, unicode: bool = True, padding: int = 1, direction: str | None = None) -> str:
    """Parse a Mermaid flowchart string and render it to ASCII/Unicode art.

    Args:
        src: Mermaid DSL source string.
        unicode: True for Unicode box-drawing characters; False for ASCII fallback.
        padding: Spaces inside node border on each side (default 1).
        direction: Override graph direction ('LR', 'RL', 'TD', 'BT'); None keeps parsed value.

    Returns:
        The rendered ASCII/Unicode string, or empty string if the graph is empty.

    Raises:
        ValueError: If the input cannot be parsed or direction is unknown.
    """
    ast_graph = parse(src)
    _apply_direction(ast_graph, direction)
    gir = GraphIR.from_ast(ast_graph)
    if gir.node_count() == 0 and not gir.subgraph_members:
        return ""
    layout_nodes, routed_edges = full_layout_with_padding(gir, padding)
    return render(gir, layout_nodes, routed_edges, unicode)


def render_dsl_padded(src: str, unicode: bool = True, padding: int = 1) -> str:
    """Parse a Mermaid flowchart string and render with a custom padding value.

    Args:
        src: Mermaid DSL source string.
        unicode: True for Unicode box-drawing characters; False for ASCII fallback.
        padding: Spaces inside node border on each side.

    Returns:
        The rendered ASCII/Unicode string, or empty string if the graph is empty.

    Raises:
        ValueError: If the input cannot be parsed.
    """
    ast_graph = parse(src)
    gir = GraphIR.from_ast(ast_graph)
    if gir.node_count() == 0 and not gir.subgraph_members:
        return ""
    layout_nodes, routed_edges = full_layout_with_padding(gir, padding)
    return render(gir, layout_nodes, routed_edges, unicode)
