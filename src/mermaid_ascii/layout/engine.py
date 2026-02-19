"""Layout engine convenience functions."""

from __future__ import annotations

from mermaid_ascii.layout.graph import GraphIR
from mermaid_ascii.layout.sugiyama import (
    NODE_PADDING,
    SugiyamaLayout,
    assign_coordinates_padded,
)
from mermaid_ascii.layout.types import LayoutResult
from mermaid_ascii.syntax.types import Direction


def full_layout(gir: GraphIR) -> LayoutResult:
    """Run the full layout pipeline with default padding."""
    return full_layout_with_padding(gir, NODE_PADDING)


def full_layout_with_padding(gir: GraphIR, padding: int) -> LayoutResult:
    """Run the default (Sugiyama) layout pipeline."""
    engine = SugiyamaLayout()
    return engine.layout(gir, padding)


def assign_coordinates(ordering, aug):
    """Assign coordinates with default padding and TD direction."""
    return assign_coordinates_padded(ordering, aug, NODE_PADDING, {}, Direction.TD)
