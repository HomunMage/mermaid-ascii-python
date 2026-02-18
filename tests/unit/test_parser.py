"""Tests for mermaid_ascii.parser — port of all 12 Rust parser tests."""

from mermaid_ascii.parsers.registry import parse
from mermaid_ascii.syntax.types import Direction, EdgeType, NodeShape


def test_parse_simple_chain():
    input = "graph TD\n    A --> B --> C\n"
    graph = parse(input)
    assert graph.direction == Direction.TD
    assert len(graph.nodes) == 3
    assert len(graph.edges) == 2
    assert graph.nodes[0].id == "A"
    assert graph.nodes[0].label == "A"
    assert graph.edges[0].from_id == "A"
    assert graph.edges[0].to_id == "B"


def test_parse_node_with_label():
    input = "graph TD\n    A[Start] --> B[End]\n"
    graph = parse(input)
    assert graph.nodes[0].id == "A"
    assert graph.nodes[0].label == "Start"
    assert graph.nodes[0].shape == NodeShape.Rectangle
    assert graph.nodes[1].id == "B"
    assert graph.nodes[1].label == "End"


def test_parse_shapes():
    input = "graph TD\n    A[Rect] --> B(Round) --> C{Diamond} --> D((Circle))\n"
    graph = parse(input)
    assert graph.nodes[0].shape == NodeShape.Rectangle
    assert graph.nodes[1].shape == NodeShape.Rounded
    assert graph.nodes[2].shape == NodeShape.Diamond
    assert graph.nodes[3].shape == NodeShape.Circle


def test_parse_edge_label():
    input = "graph TD\n    A -->|yes| B\n"
    graph = parse(input)
    assert graph.edges[0].label == "yes"


def test_parse_flowchart_keyword():
    input = "flowchart LR\n    A --> B\n"
    graph = parse(input)
    assert graph.direction == Direction.LR


def test_parse_subgraph():
    input = "graph TD\n    subgraph Group\n        A --> B\n    end\n"
    graph = parse(input)
    assert len(graph.subgraphs) == 1
    assert graph.subgraphs[0].name == "Group"
    assert len(graph.subgraphs[0].nodes) == 2
    assert len(graph.subgraphs[0].edges) == 1


def test_first_definition_wins():
    input = "graph TD\n    A[Hello] --> B\n    A[World] --> C\n"
    graph = parse(input)
    a_node = next(n for n in graph.nodes if n.id == "A")
    assert a_node.label == "Hello"


def test_parse_no_header():
    input = "A --> B\n"
    graph = parse(input)
    assert graph.direction == Direction.TD  # default
    assert len(graph.nodes) == 2


def test_parse_comments():
    input = "graph TD\n    %% This is a comment\n    A --> B\n"
    graph = parse(input)
    assert len(graph.nodes) == 2


def test_parse_edge_types():
    input = "graph TD\n    A --> B\n    C --- D\n    E -.-> F\n    G ==> H\n"
    graph = parse(input)
    assert graph.edges[0].edge_type == EdgeType.Arrow
    assert graph.edges[1].edge_type == EdgeType.Line
    assert graph.edges[2].edge_type == EdgeType.DottedArrow
    assert graph.edges[3].edge_type == EdgeType.ThickArrow


def test_parse_quoted_label():
    input = 'graph TD\n    A["Hello World"] --> B\n'
    graph = parse(input)
    assert graph.nodes[0].label == "Hello World"


def test_parse_multiline_label():
    input = 'graph TD\n    A["Line1\\nLine2"] --> B\n'
    graph = parse(input)
    assert graph.nodes[0].label == "Line1\nLine2"
