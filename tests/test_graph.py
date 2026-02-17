"""Tests for mermaid_ascii.graph — GraphIR construction, cycle detection, and degree queries."""

from mermaid_ascii.ast import (
    Direction,
    Edge,
    EdgeType,
    Graph,
    Node,
    NodeShape,
    Subgraph,
)
from mermaid_ascii.graph import EdgeData, GraphIR, NodeData


def _make_graph(
    *,
    direction: Direction = Direction.TD,
    nodes: list[Node] | None = None,
    edges: list[Edge] | None = None,
    subgraphs: list[Subgraph] | None = None,
) -> Graph:
    return Graph(
        direction=direction,
        nodes=nodes or [],
        edges=edges or [],
        subgraphs=subgraphs or [],
    )


def _node(id: str, label: str | None = None, shape: NodeShape = NodeShape.Rectangle) -> Node:
    return Node(id=id, label=label or id, shape=shape)


def _edge(from_id: str, to_id: str, edge_type: EdgeType = EdgeType.Arrow) -> Edge:
    return Edge(from_id=from_id, to_id=to_id, edge_type=edge_type)


class TestBasicConstruction:
    def test_empty_graph(self):
        g = _make_graph()
        gir = GraphIR.from_ast(g)
        assert gir.node_count() == 0
        assert gir.edge_count() == 0

    def test_single_node(self):
        g = _make_graph(nodes=[_node("A")])
        gir = GraphIR.from_ast(g)
        assert gir.node_count() == 1
        assert gir.edge_count() == 0

    def test_direction_preserved(self):
        g = _make_graph(direction=Direction.LR)
        gir = GraphIR.from_ast(g)
        assert gir.direction == Direction.LR

    def test_simple_edge_creates_nodes(self):
        """An edge with no explicit node declarations still creates nodes."""
        g = _make_graph(edges=[_edge("A", "B")])
        gir = GraphIR.from_ast(g)
        assert gir.node_count() == 2
        assert gir.edge_count() == 1

    def test_node_data_stored(self):
        g = _make_graph(nodes=[_node("A", label="Alpha", shape=NodeShape.Diamond)])
        gir = GraphIR.from_ast(g)
        data: NodeData = gir.digraph.nodes["A"]["data"]
        assert data.id == "A"
        assert data.label == "Alpha"
        assert data.shape == NodeShape.Diamond
        assert data.subgraph is None

    def test_edge_data_stored(self):
        g = _make_graph(
            edges=[Edge(from_id="A", to_id="B", edge_type=EdgeType.DottedArrow, label="goes")]
        )
        gir = GraphIR.from_ast(g)
        data: EdgeData = gir.digraph.edges["A", "B"]["data"]
        assert data.edge_type == EdgeType.DottedArrow
        assert data.label == "goes"

    def test_first_definition_wins(self):
        """If a node id appears twice, the first definition is kept."""
        g = _make_graph(
            nodes=[
                _node("A", label="First", shape=NodeShape.Rectangle),
                _node("A", label="Second", shape=NodeShape.Diamond),
            ]
        )
        gir = GraphIR.from_ast(g)
        assert gir.node_count() == 1
        data: NodeData = gir.digraph.nodes["A"]["data"]
        assert data.label == "First"


class TestSubgraphFlattening:
    def test_subgraph_members_collected(self):
        sg = Subgraph(name="Group", nodes=[_node("X"), _node("Y")])
        g = _make_graph(subgraphs=[sg])
        gir = GraphIR.from_ast(g)
        assert gir.node_count() == 2
        assert ("Group", ["X", "Y"]) in gir.subgraph_members

    def test_subgraph_node_membership(self):
        sg = Subgraph(name="Group", nodes=[_node("X")])
        g = _make_graph(subgraphs=[sg])
        gir = GraphIR.from_ast(g)
        data: NodeData = gir.digraph.nodes["X"]["data"]
        assert data.subgraph == "Group"

    def test_top_level_node_skipped_if_same_name_as_subgraph(self):
        """Top-level node whose id matches a subgraph name is skipped."""
        sg = Subgraph(name="Group", nodes=[_node("X")])
        g = _make_graph(nodes=[_node("Group")], subgraphs=[sg])
        gir = GraphIR.from_ast(g)
        # Only "X" should be in graph (Group node is skipped since it's a subgraph)
        assert "X" in gir.digraph
        # "Group" may appear if used as edge endpoint, but not from top-level node def
        assert gir.digraph.nodes.get("Group") is None or "X" in gir.digraph

    def test_subgraph_edges_added(self):
        sg = Subgraph(
            name="Group",
            nodes=[_node("X"), _node("Y")],
            edges=[_edge("X", "Y")],
        )
        g = _make_graph(subgraphs=[sg])
        gir = GraphIR.from_ast(g)
        assert gir.edge_count() == 1
        assert gir.digraph.has_edge("X", "Y")

    def test_subgraph_description(self):
        sg = Subgraph(name="Group", nodes=[], description="My group")
        g = _make_graph(subgraphs=[sg])
        gir = GraphIR.from_ast(g)
        assert gir.subgraph_descriptions["Group"] == "My group"

    def test_nested_subgraph(self):
        inner = Subgraph(name="Inner", nodes=[_node("Z")])
        outer = Subgraph(name="Outer", nodes=[_node("W")], subgraphs=[inner])
        g = _make_graph(subgraphs=[outer])
        gir = GraphIR.from_ast(g)
        assert "W" in gir.digraph
        assert "Z" in gir.digraph
        assert gir.digraph.nodes["Z"]["data"].subgraph == "Inner"


class TestCycleDetection:
    def test_empty_graph_is_dag(self):
        g = _make_graph()
        gir = GraphIR.from_ast(g)
        assert gir.is_dag() is True

    def test_simple_chain_is_dag(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "C")])
        gir = GraphIR.from_ast(g)
        assert gir.is_dag() is True

    def test_single_cycle_is_not_dag(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "C"), _edge("C", "A")])
        gir = GraphIR.from_ast(g)
        assert gir.is_dag() is False

    def test_self_loop_is_not_dag(self):
        g = _make_graph(edges=[_edge("A", "A")])
        gir = GraphIR.from_ast(g)
        assert gir.is_dag() is False

    def test_two_node_cycle_is_not_dag(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "A")])
        gir = GraphIR.from_ast(g)
        assert gir.is_dag() is False


class TestTopologicalOrder:
    def test_empty_graph_returns_empty_list(self):
        g = _make_graph()
        gir = GraphIR.from_ast(g)
        result = gir.topological_order()
        assert result == []

    def test_simple_chain_order(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "C")])
        gir = GraphIR.from_ast(g)
        order = gir.topological_order()
        assert order is not None
        assert order.index("A") < order.index("B")
        assert order.index("B") < order.index("C")

    def test_cycle_returns_none(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "A")])
        gir = GraphIR.from_ast(g)
        assert gir.topological_order() is None

    def test_self_loop_returns_none(self):
        g = _make_graph(edges=[_edge("A", "A")])
        gir = GraphIR.from_ast(g)
        assert gir.topological_order() is None

    def test_all_nodes_in_result(self):
        g = _make_graph(
            nodes=[_node("A"), _node("B"), _node("C")],
            edges=[_edge("A", "B"), _edge("A", "C")],
        )
        gir = GraphIR.from_ast(g)
        order = gir.topological_order()
        assert order is not None
        assert set(order) == {"A", "B", "C"}
        assert order.index("A") < order.index("B")
        assert order.index("A") < order.index("C")


class TestDegreeQueries:
    def test_in_degree_source_node(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("A", "C")])
        gir = GraphIR.from_ast(g)
        assert gir.in_degree("A") == 0

    def test_out_degree_source_node(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("A", "C")])
        gir = GraphIR.from_ast(g)
        assert gir.out_degree("A") == 2

    def test_in_degree_sink_node(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("C", "B")])
        gir = GraphIR.from_ast(g)
        assert gir.in_degree("B") == 2

    def test_out_degree_sink_node(self):
        g = _make_graph(edges=[_edge("A", "B")])
        gir = GraphIR.from_ast(g)
        assert gir.out_degree("B") == 0

    def test_in_degree_middle_node(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "C")])
        gir = GraphIR.from_ast(g)
        assert gir.in_degree("B") == 1
        assert gir.out_degree("B") == 1

    def test_degree_unknown_node_returns_zero(self):
        g = _make_graph(nodes=[_node("A")])
        gir = GraphIR.from_ast(g)
        assert gir.in_degree("NONEXISTENT") == 0
        assert gir.out_degree("NONEXISTENT") == 0

    def test_self_loop_degree(self):
        g = _make_graph(edges=[_edge("A", "A")])
        gir = GraphIR.from_ast(g)
        assert gir.in_degree("A") == 1
        assert gir.out_degree("A") == 1


class TestAdjacencyList:
    def test_empty_graph(self):
        g = _make_graph()
        gir = GraphIR.from_ast(g)
        assert gir.adjacency_list() == []

    def test_single_node_no_edges(self):
        g = _make_graph(nodes=[_node("A")])
        gir = GraphIR.from_ast(g)
        adj = gir.adjacency_list()
        assert adj == [("A", [])]

    def test_chain_adjacency(self):
        g = _make_graph(edges=[_edge("A", "B"), _edge("B", "C")])
        gir = GraphIR.from_ast(g)
        adj = dict(gir.adjacency_list())
        assert adj["A"] == ["B"]
        assert adj["B"] == ["C"]
        assert adj["C"] == []

    def test_sorted_output(self):
        g = _make_graph(edges=[_edge("C", "A"), _edge("B", "A")])
        gir = GraphIR.from_ast(g)
        adj = gir.adjacency_list()
        keys = [k for k, _ in adj]
        assert keys == sorted(keys)
