use super::*;
use crate::syntax::types::{Edge, EdgeType, Graph, Node, NodeShape, Subgraph};

fn make_graph(
    direction: Direction,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    subgraphs: Vec<Subgraph>,
) -> Graph {
    Graph {
        direction,
        nodes,
        edges,
        subgraphs,
    }
}

fn node(id: &str) -> Node {
    Node::bare(id)
}

fn node_labeled(id: &str, label: &str, shape: NodeShape) -> Node {
    Node::new(id, label, shape)
}

fn edge(from: &str, to: &str) -> Edge {
    Edge::new(from, to, EdgeType::Arrow)
}

fn edge_typed(from: &str, to: &str, et: EdgeType) -> Edge {
    Edge::new(from, to, et)
}

// ── Basic construction ────────────────────────────────────────────────────

#[test]
fn test_empty_graph() {
    let g = make_graph(Direction::TD, vec![], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 0);
    assert_eq!(gir.edge_count(), 0);
}

#[test]
fn test_single_node() {
    let g = make_graph(Direction::TD, vec![node("A")], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 1);
    assert_eq!(gir.edge_count(), 0);
}

#[test]
fn test_direction_preserved() {
    let g = make_graph(Direction::LR, vec![], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.direction, Direction::LR);
}

#[test]
fn test_simple_edge_creates_nodes() {
    let g = make_graph(Direction::TD, vec![], vec![edge("A", "B")], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 2);
    assert_eq!(gir.edge_count(), 1);
}

#[test]
fn test_node_data_stored() {
    let g = make_graph(
        Direction::TD,
        vec![node_labeled("A", "Alpha", NodeShape::Diamond)],
        vec![],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    let idx = gir.node_index["A"];
    let data = &gir.digraph[idx];
    assert_eq!(data.id, "A");
    assert_eq!(data.label, "Alpha");
    assert_eq!(data.shape, NodeShape::Diamond);
    assert!(data.subgraph.is_none());
}

#[test]
fn test_edge_data_stored() {
    let mut e = Edge::new("A", "B", EdgeType::DottedArrow);
    e.label = Some("goes".to_string());
    let g = make_graph(Direction::TD, vec![], vec![e], vec![]);
    let gir = GraphIR::from_ast(&g);
    let from_idx = gir.node_index["A"];
    let to_idx = gir.node_index["B"];
    let edge_idx = gir.digraph.find_edge(from_idx, to_idx).unwrap();
    let data = &gir.digraph[edge_idx];
    assert_eq!(data.edge_type, EdgeType::DottedArrow);
    assert_eq!(data.label.as_deref(), Some("goes"));
}

#[test]
fn test_first_definition_wins() {
    let g = make_graph(
        Direction::TD,
        vec![
            node_labeled("A", "First", NodeShape::Rectangle),
            node_labeled("A", "Second", NodeShape::Diamond),
        ],
        vec![],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 1);
    let idx = gir.node_index["A"];
    assert_eq!(gir.digraph[idx].label, "First");
}

// ── Subgraph flattening ───────────────────────────────────────────────────

#[test]
fn test_subgraph_members_collected() {
    let mut sg = Subgraph::new("Group");
    sg.nodes = vec![node("X"), node("Y")];
    let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 2);
    assert!(
        gir.subgraph_members
            .contains(&("Group".to_string(), vec!["X".to_string(), "Y".to_string()]))
    );
}

#[test]
fn test_subgraph_node_membership() {
    let mut sg = Subgraph::new("Group");
    sg.nodes = vec![node("X")];
    let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
    let gir = GraphIR::from_ast(&g);
    let idx = gir.node_index["X"];
    assert_eq!(gir.digraph[idx].subgraph.as_deref(), Some("Group"));
}

#[test]
fn test_top_level_node_skipped_if_same_name_as_subgraph() {
    let mut sg = Subgraph::new("Group");
    sg.nodes = vec![node("X")];
    let g = make_graph(Direction::TD, vec![node("Group")], vec![], vec![sg]);
    let gir = GraphIR::from_ast(&g);
    // "X" must be present
    assert!(gir.node_index.contains_key("X"));
}

#[test]
fn test_subgraph_edges_added() {
    let mut sg = Subgraph::new("Group");
    sg.nodes = vec![node("X"), node("Y")];
    sg.edges = vec![edge("X", "Y")];
    let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.edge_count(), 1);
    let x = gir.node_index["X"];
    let y = gir.node_index["Y"];
    assert!(gir.digraph.find_edge(x, y).is_some());
}

#[test]
fn test_subgraph_description() {
    let mut sg = Subgraph::new("Group");
    sg.description = Some("My group".to_string());
    let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(
        gir.subgraph_descriptions.get("Group").map(|s| s.as_str()),
        Some("My group")
    );
}

#[test]
fn test_nested_subgraph() {
    let mut inner = Subgraph::new("Inner");
    inner.nodes = vec![node("Z")];
    let mut outer = Subgraph::new("Outer");
    outer.nodes = vec![node("W")];
    outer.subgraphs = vec![inner];
    let g = make_graph(Direction::TD, vec![], vec![], vec![outer]);
    let gir = GraphIR::from_ast(&g);
    assert!(gir.node_index.contains_key("W"));
    assert!(gir.node_index.contains_key("Z"));
    let z_idx = gir.node_index["Z"];
    assert_eq!(gir.digraph[z_idx].subgraph.as_deref(), Some("Inner"));
}

// ── Cycle detection ───────────────────────────────────────────────────────

#[test]
fn test_empty_graph_is_dag() {
    let g = make_graph(Direction::TD, vec![], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert!(gir.is_dag());
}

#[test]
fn test_simple_chain_is_dag() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("B", "C")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert!(gir.is_dag());
}

#[test]
fn test_single_cycle_is_not_dag() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("B", "C"), edge("C", "A")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert!(!gir.is_dag());
}

#[test]
fn test_self_loop_is_not_dag() {
    let g = make_graph(Direction::TD, vec![], vec![edge("A", "A")], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert!(!gir.is_dag());
}

#[test]
fn test_two_node_cycle_is_not_dag() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("B", "A")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert!(!gir.is_dag());
}

// ── Topological order ─────────────────────────────────────────────────────

#[test]
fn test_empty_graph_topo_returns_empty() {
    let g = make_graph(Direction::TD, vec![], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    let result = gir.topological_order();
    assert_eq!(result, Some(vec![]));
}

#[test]
fn test_simple_chain_topo_order() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("B", "C")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    let order = gir.topological_order().unwrap();
    let a_pos = order.iter().position(|x| x == "A").unwrap();
    let b_pos = order.iter().position(|x| x == "B").unwrap();
    let c_pos = order.iter().position(|x| x == "C").unwrap();
    assert!(a_pos < b_pos);
    assert!(b_pos < c_pos);
}

#[test]
fn test_cycle_topo_returns_none() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("B", "A")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert!(gir.topological_order().is_none());
}

#[test]
fn test_self_loop_topo_returns_none() {
    let g = make_graph(Direction::TD, vec![], vec![edge("A", "A")], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert!(gir.topological_order().is_none());
}

// ── Degree queries ────────────────────────────────────────────────────────

#[test]
fn test_in_degree_source_node() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("A", "C")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.in_degree("A"), 0);
}

#[test]
fn test_out_degree_source_node() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("A", "C")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.out_degree("A"), 2);
}

#[test]
fn test_in_degree_sink_node() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("C", "B")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.in_degree("B"), 2);
}

#[test]
fn test_out_degree_sink_node() {
    let g = make_graph(Direction::TD, vec![], vec![edge("A", "B")], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.out_degree("B"), 0);
}

#[test]
fn test_degree_unknown_node_returns_zero() {
    let g = make_graph(Direction::TD, vec![node("A")], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.in_degree("NONEXISTENT"), 0);
    assert_eq!(gir.out_degree("NONEXISTENT"), 0);
}

#[test]
fn test_self_loop_degree() {
    let g = make_graph(Direction::TD, vec![], vec![edge("A", "A")], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.in_degree("A"), 1);
    assert_eq!(gir.out_degree("A"), 1);
}

// ── Adjacency list ────────────────────────────────────────────────────────

#[test]
fn test_empty_adjacency_list() {
    let g = make_graph(Direction::TD, vec![], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.adjacency_list(), vec![]);
}

#[test]
fn test_single_node_adjacency() {
    let g = make_graph(Direction::TD, vec![node("A")], vec![], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.adjacency_list(), vec![("A".to_string(), vec![])]);
}

#[test]
fn test_chain_adjacency() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("B", "C")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    let adj: HashMap<String, Vec<String>> = gir.adjacency_list().into_iter().collect();
    assert_eq!(adj["A"], vec!["B"]);
    assert_eq!(adj["B"], vec!["C"]);
    assert_eq!(adj["C"], Vec::<String>::new());
}

#[test]
fn test_neighbors_sorted() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "C"), edge("A", "B")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    let adj: HashMap<String, Vec<String>> = gir.adjacency_list().into_iter().collect();
    assert_eq!(adj["A"], vec!["B", "C"]);
}

#[test]
fn test_adjacency_list_sorted_by_key() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("C", "A"), edge("B", "A")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    let keys: Vec<String> = gir.adjacency_list().into_iter().map(|(k, _)| k).collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

// ── Edge types ────────────────────────────────────────────────────────────

#[test]
fn test_all_edge_types_stored() {
    let types = vec![
        EdgeType::Arrow,
        EdgeType::Line,
        EdgeType::DottedArrow,
        EdgeType::DottedLine,
        EdgeType::ThickArrow,
        EdgeType::ThickLine,
        EdgeType::BidirArrow,
        EdgeType::BidirDotted,
        EdgeType::BidirThick,
    ];
    for et in types {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge_typed("A", "B", et.clone())],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        let from_idx = gir.node_index["A"];
        let to_idx = gir.node_index["B"];
        let eidx = gir.digraph.find_edge(from_idx, to_idx).unwrap();
        assert_eq!(gir.digraph[eidx].edge_type, et);
    }
}

// ── Node / edge count ─────────────────────────────────────────────────────

#[test]
fn test_no_duplicate_nodes_from_shared_edge_endpoint() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![edge("A", "B"), edge("A", "C")],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 3);
}

#[test]
fn test_explicit_and_implicit_same_node() {
    let g = make_graph(Direction::TD, vec![node("A")], vec![edge("A", "B")], vec![]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 2);
}

#[test]
fn test_diamond_graph() {
    let g = make_graph(
        Direction::TD,
        vec![],
        vec![
            edge("A", "B"),
            edge("A", "C"),
            edge("B", "D"),
            edge("C", "D"),
        ],
        vec![],
    );
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 4);
    assert_eq!(gir.edge_count(), 4);
    assert!(gir.is_dag());
}

// ── Extended subgraph ─────────────────────────────────────────────────────

#[test]
fn test_multiple_subgraphs_members() {
    let mut sg1 = Subgraph::new("SG1");
    sg1.nodes = vec![node("A"), node("B")];
    let mut sg2 = Subgraph::new("SG2");
    sg2.nodes = vec![node("C")];
    let g = make_graph(Direction::TD, vec![], vec![], vec![sg1, sg2]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 3);
    assert_eq!(gir.subgraph_members.len(), 2);
    let names: HashSet<&str> = gir
        .subgraph_members
        .iter()
        .map(|(n, _)| n.as_str())
        .collect();
    assert!(names.contains("SG1"));
    assert!(names.contains("SG2"));
}

#[test]
fn test_cross_subgraph_edge_at_top_level() {
    let mut sg1 = Subgraph::new("SG1");
    sg1.nodes = vec![node("A")];
    let mut sg2 = Subgraph::new("SG2");
    sg2.nodes = vec![node("B")];
    let g = make_graph(Direction::TD, vec![], vec![edge("A", "B")], vec![sg1, sg2]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.edge_count(), 1);
    let a = gir.node_index["A"];
    let b = gir.node_index["B"];
    assert!(gir.digraph.find_edge(a, b).is_some());
}

#[test]
fn test_deeply_nested_subgraph() {
    let mut innermost = Subgraph::new("Level3");
    innermost.nodes = vec![node("P")];
    let mut middle = Subgraph::new("Level2");
    middle.nodes = vec![node("Q")];
    middle.subgraphs = vec![innermost];
    let mut outer = Subgraph::new("Level1");
    outer.nodes = vec![node("R")];
    outer.subgraphs = vec![middle];
    let g = make_graph(Direction::TD, vec![], vec![], vec![outer]);
    let gir = GraphIR::from_ast(&g);
    assert_eq!(gir.node_count(), 3);
    assert_eq!(gir.subgraph_members.len(), 3);
    let names: HashSet<&str> = gir
        .subgraph_members
        .iter()
        .map(|(n, _)| n.as_str())
        .collect();
    assert!(names.contains("Level1"));
    assert!(names.contains("Level2"));
    assert!(names.contains("Level3"));
}

#[test]
fn test_no_description_when_none() {
    let sg = Subgraph::new("SG");
    let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
    let gir = GraphIR::from_ast(&g);
    assert!(!gir.subgraph_descriptions.contains_key("SG"));
}
