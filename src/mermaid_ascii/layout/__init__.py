"""Layout engine registry and public API."""

from __future__ import annotations

from mermaid_ascii.ir.graph import GraphIR
from mermaid_ascii.layout.sugiyama import (
    COMPOUND_PREFIX,
    H_GAP,
    NODE_HEIGHT,
    NODE_PADDING,
    V_GAP,
    AugmentedGraph,
    CompoundInfo,
    CycleRemovalResult,
    DummyEdge,
    LayerAssignment,
    SugiyamaLayout,
    assign_coordinates_padded,
    collapse_subgraphs,
    compute_compound_dimensions,
    count_crossings,
    expand_compound_nodes,
    greedy_fas_ordering,
    insert_dummy_nodes,
    label_dimensions,
    minimise_crossings,
    remove_cycles,
    route_edges,
)
from mermaid_ascii.layout.types import DUMMY_PREFIX, LayoutNode, Point, RoutedEdge

__all__ = [
    "COMPOUND_PREFIX",
    "DUMMY_PREFIX",
    "H_GAP",
    "NODE_HEIGHT",
    "NODE_PADDING",
    "V_GAP",
    "AugmentedGraph",
    "CompoundInfo",
    "CycleRemovalResult",
    "DummyEdge",
    "LayerAssignment",
    "LayoutNode",
    "Point",
    "RoutedEdge",
    "SugiyamaLayout",
    "assign_coordinates_padded",
    "collapse_subgraphs",
    "compute_compound_dimensions",
    "count_crossings",
    "expand_compound_nodes",
    "full_layout",
    "full_layout_with_padding",
    "greedy_fas_ordering",
    "insert_dummy_nodes",
    "label_dimensions",
    "minimise_crossings",
    "remove_cycles",
    "route_edges",
]


def full_layout(gir: GraphIR) -> tuple[list[LayoutNode], list[RoutedEdge]]:
    """Run the full layout pipeline with default padding."""
    return full_layout_with_padding(gir, NODE_PADDING)


def full_layout_with_padding(gir: GraphIR, padding: int) -> tuple[list[LayoutNode], list[RoutedEdge]]:
    """Run the default (Sugiyama) layout pipeline."""
    engine = SugiyamaLayout()
    return engine.layout(gir, padding)


def assign_coordinates(ordering, aug):
    """Backward-compat wrapper."""
    from mermaid_ascii.types import Direction

    return assign_coordinates_padded(ordering, aug, NODE_PADDING, {}, Direction.TD)


def compute_orthogonal_waypoints(from_node, to_node, layer_top_y, layer_bottom_y, dummy_xs):
    """Backward-compat wrapper."""
    from mermaid_ascii.layout.sugiyama import _compute_orthogonal_waypoints

    return _compute_orthogonal_waypoints(from_node, to_node, layer_top_y, layer_bottom_y, dummy_xs)
