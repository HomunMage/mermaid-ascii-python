"""Shared type definitions for mermaid-ascii.

Enums and small types used across parsers, IR, layout, and renderers.
"""

from __future__ import annotations

from enum import Enum, auto


class Direction(Enum):
    LR = auto()
    RL = auto()
    TD = auto()
    BT = auto()

    @classmethod
    def default(cls) -> Direction:
        return cls.TD


class NodeShape(Enum):
    Rectangle = auto()  # id[Label]
    Rounded = auto()  # id(Label)
    Diamond = auto()  # id{Label}
    Circle = auto()  # id((Label))

    @classmethod
    def default(cls) -> NodeShape:
        return cls.Rectangle


class EdgeType(Enum):
    Arrow = auto()  # -->
    Line = auto()  # ---
    DottedArrow = auto()  # -.->
    DottedLine = auto()  # -.-
    ThickArrow = auto()  # ==>
    ThickLine = auto()  # ===
    BidirArrow = auto()  # <-->
    BidirDotted = auto()  # <-.->
    BidirThick = auto()  # <==>
