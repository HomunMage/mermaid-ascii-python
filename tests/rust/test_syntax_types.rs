use super::*;

#[test]
fn test_direction_default() {
    assert_eq!(Direction::default(), Direction::TD);
}

#[test]
fn test_node_shape_default() {
    assert_eq!(NodeShape::default(), NodeShape::Rectangle);
}

#[test]
fn test_node_new() {
    let n = Node::new("A", "Hello", NodeShape::Rounded);
    assert_eq!(n.id, "A");
    assert_eq!(n.label, "Hello");
    assert_eq!(n.shape, NodeShape::Rounded);
    assert!(n.attrs.is_empty());
}

#[test]
fn test_node_bare() {
    let n = Node::bare("B");
    assert_eq!(n.id, "B");
    assert_eq!(n.label, "B");
    assert_eq!(n.shape, NodeShape::Rectangle);
}

#[test]
fn test_edge_new() {
    let e = Edge::new("A", "B", EdgeType::Arrow);
    assert_eq!(e.from_id, "A");
    assert_eq!(e.to_id, "B");
    assert_eq!(e.edge_type, EdgeType::Arrow);
    assert!(e.label.is_none());
}

#[test]
fn test_subgraph_new() {
    let sg = Subgraph::new("Group");
    assert_eq!(sg.name, "Group");
    assert!(sg.nodes.is_empty());
    assert!(sg.edges.is_empty());
    assert!(sg.direction.is_none());
}

#[test]
fn test_graph_new() {
    let g = Graph::new();
    assert_eq!(g.direction, Direction::TD);
    assert!(g.nodes.is_empty());
    assert!(g.edges.is_empty());
    assert!(g.subgraphs.is_empty());
}

#[test]
fn test_edge_with_label() {
    let mut e = Edge::new("X", "Y", EdgeType::DottedArrow);
    e.label = Some("my label".to_string());
    assert_eq!(e.label.as_deref(), Some("my label"));
}
