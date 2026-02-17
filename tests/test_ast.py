"""Tests for mermaid_ascii.ast module.

Port of Rust ast tests. Verifies construction, defaults, and helper methods.
"""

from mermaid_ascii.ast import (
    Attr,
    Direction,
    Edge,
    EdgeType,
    Graph,
    Node,
    NodeShape,
    Subgraph,
)


# ─── Direction ───────────────────────────────────────────────────────────────

def test_direction_default():
    graph = Graph()
    assert graph.direction == Direction.TD


def test_direction_variants():
    assert Direction.LR != Direction.RL
    assert Direction.TD != Direction.BT
    assert len(Direction) == 4


# ─── NodeShape ───────────────────────────────────────────────────────────────

def test_node_shape_variants():
    assert len(NodeShape) == 4
    assert NodeShape.Rectangle != NodeShape.Rounded
    assert NodeShape.Diamond != NodeShape.Circle


# ─── EdgeType ────────────────────────────────────────────────────────────────

def test_edge_type_variants():
    assert len(EdgeType) == 9
    types = [
        EdgeType.Arrow,
        EdgeType.Line,
        EdgeType.DottedArrow,
        EdgeType.DottedLine,
        EdgeType.ThickArrow,
        EdgeType.ThickLine,
        EdgeType.BidirArrow,
        EdgeType.BidirDotted,
        EdgeType.BidirThick,
    ]
    assert len(set(types)) == 9


# ─── Attr ────────────────────────────────────────────────────────────────────

def test_attr_construction():
    attr = Attr(key="color", value="red")
    assert attr.key == "color"
    assert attr.value == "red"


def test_attr_equality():
    a = Attr(key="k", value="v")
    b = Attr(key="k", value="v")
    assert a == b


# ─── Node ────────────────────────────────────────────────────────────────────

def test_node_construction():
    node = Node(id="A", label="Hello", shape=NodeShape.Rectangle)
    assert node.id == "A"
    assert node.label == "Hello"
    assert node.shape == NodeShape.Rectangle
    assert node.attrs == []


def test_node_bare():
    node = Node.bare("MyNode")
    assert node.id == "MyNode"
    assert node.label == "MyNode"
    assert node.shape == NodeShape.Rectangle
    assert node.attrs == []


def test_node_new_classmethod():
    node = Node.new("B", "Box", NodeShape.Rounded)
    assert node.id == "B"
    assert node.label == "Box"
    assert node.shape == NodeShape.Rounded


def test_node_with_attrs():
    attrs = [Attr(key="style", value="fill:#f9f")]
    node = Node(id="C", label="C", shape=NodeShape.Diamond, attrs=attrs)
    assert len(node.attrs) == 1
    assert node.attrs[0].key == "style"


def test_node_equality():
    n1 = Node.bare("X")
    n2 = Node.bare("X")
    assert n1 == n2
    n3 = Node.bare("Y")
    assert n1 != n3


def test_node_shapes():
    shapes = [NodeShape.Rectangle, NodeShape.Rounded, NodeShape.Diamond, NodeShape.Circle]
    for shape in shapes:
        node = Node(id="n", label="n", shape=shape)
        assert node.shape == shape


# ─── Edge ────────────────────────────────────────────────────────────────────

def test_edge_construction():
    edge = Edge(from_id="A", to_id="B", edge_type=EdgeType.Arrow)
    assert edge.from_id == "A"
    assert edge.to_id == "B"
    assert edge.edge_type == EdgeType.Arrow
    assert edge.label is None
    assert edge.attrs == []


def test_edge_new_classmethod():
    edge = Edge.new("X", "Y", EdgeType.DottedArrow)
    assert edge.from_id == "X"
    assert edge.to_id == "Y"
    assert edge.edge_type == EdgeType.DottedArrow
    assert edge.label is None


def test_edge_with_label():
    edge = Edge(from_id="A", to_id="B", edge_type=EdgeType.ThickArrow, label="yes")
    assert edge.label == "yes"


def test_edge_equality():
    e1 = Edge.new("A", "B", EdgeType.Arrow)
    e2 = Edge.new("A", "B", EdgeType.Arrow)
    assert e1 == e2
    e3 = Edge.new("A", "B", EdgeType.Line)
    assert e1 != e3


# ─── Subgraph ────────────────────────────────────────────────────────────────

def test_subgraph_construction():
    sg = Subgraph(name="Group1")
    assert sg.name == "Group1"
    assert sg.nodes == []
    assert sg.edges == []
    assert sg.subgraphs == []
    assert sg.description is None
    assert sg.direction is None


def test_subgraph_new_classmethod():
    sg = Subgraph.new("MyGroup")
    assert sg.name == "MyGroup"
    assert sg.nodes == []


def test_subgraph_with_direction():
    sg = Subgraph(name="G", direction=Direction.LR)
    assert sg.direction == Direction.LR


def test_subgraph_with_description():
    sg = Subgraph(name="G", description="A group of nodes")
    assert sg.description == "A group of nodes"


def test_subgraph_nested():
    inner = Subgraph.new("inner")
    outer = Subgraph(name="outer", subgraphs=[inner])
    assert len(outer.subgraphs) == 1
    assert outer.subgraphs[0].name == "inner"


# ─── Graph ───────────────────────────────────────────────────────────────────

def test_graph_default():
    g = Graph()
    assert g.direction == Direction.TD
    assert g.nodes == []
    assert g.edges == []
    assert g.subgraphs == []


def test_graph_with_direction():
    g = Graph(direction=Direction.LR)
    assert g.direction == Direction.LR


def test_graph_with_nodes_and_edges():
    nodes = [Node.bare("A"), Node.bare("B")]
    edges = [Edge.new("A", "B", EdgeType.Arrow)]
    g = Graph(nodes=nodes, edges=edges)
    assert len(g.nodes) == 2
    assert len(g.edges) == 1
    assert g.edges[0].from_id == "A"
    assert g.edges[0].to_id == "B"


def test_graph_with_subgraph():
    sg = Subgraph.new("S1")
    g = Graph(subgraphs=[sg])
    assert len(g.subgraphs) == 1
    assert g.subgraphs[0].name == "S1"


def test_graph_independent_defaults():
    """Each Graph() instance gets its own independent mutable collections."""
    g1 = Graph()
    g2 = Graph()
    g1.nodes.append(Node.bare("X"))
    assert g2.nodes == [], "Mutable default lists must be independent"
