"""Layout types shared across layout engines and renderers."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass
class LayoutNode:
    """A positioned node in the layout."""

    id: str
    layer: int
    order: int
    x: int
    y: int
    width: int
    height: int


@dataclass
class Point:
    """A 2D point in character coordinates (column, row)."""

    x: int
    y: int


@dataclass
class RoutedEdge:
    """A routed edge with orthogonal waypoints."""

    from_id: str
    to_id: str
    label: str | None
    edge_type: object  # EdgeType
    waypoints: list[Point]


# Prefix constants
DUMMY_PREFIX = "__dummy_"
COMPOUND_PREFIX = "__sg_"
