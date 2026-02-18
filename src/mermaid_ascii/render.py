"""Backward-compat re-exports from renderers."""

from mermaid_ascii.renderers.ascii import (  # noqa: F401
    AsciiRenderer,
    flip_horizontal,
    flip_vertical,
    remap_char_horizontal,
    remap_char_vertical,
)
from mermaid_ascii.renderers.canvas import Canvas, Rect  # noqa: F401
from mermaid_ascii.renderers.charset import Arms, BoxChars, CharSet  # noqa: F401


def render(gir, layout_nodes, routed_edges, unicode=True):
    """Backward-compat render function."""
    renderer = AsciiRenderer(unicode=unicode)
    return renderer.render(gir, layout_nodes, routed_edges)
