"""Parser registry — auto-detect diagram type and dispatch to the right parser."""

from __future__ import annotations

from mermaid_ascii.ir.ast import Graph
from mermaid_ascii.parsers.flowchart import FlowchartParser


def detect_type(src: str) -> str:
    """Detect the diagram type from source text. Returns 'flowchart' etc."""
    stripped = src.strip()
    for line in stripped.split("\n"):
        line = line.strip()
        if not line or line.startswith("%%"):
            continue
        lower = line.lower()
        if lower.startswith("flowchart") or lower.startswith("graph"):
            return "flowchart"
        # Future: sequence, class, er, etc.
        break
    return "flowchart"  # default


_PARSERS = {
    "flowchart": FlowchartParser,
}


def parse(src: str) -> Graph:
    """Auto-detect diagram type and parse to AST."""
    diagram_type = detect_type(src)
    parser_cls = _PARSERS.get(diagram_type)
    if parser_cls is None:
        raise ValueError(f"Unsupported diagram type: {diagram_type}")
    return parser_cls().parse(src)
