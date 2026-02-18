use super::*;
use crate::syntax::types::EdgeType;

#[test]
fn test_point_new() {
    let p = Point::new(3, 7);
    assert_eq!(p.x, 3);
    assert_eq!(p.y, 7);
}

#[test]
fn test_layout_node_new() {
    let n = LayoutNode::new("A", 0, 1, 10, 20, 5, 3);
    assert_eq!(n.id, "A");
    assert_eq!(n.layer, 0);
    assert_eq!(n.order, 1);
    assert_eq!(n.x, 10);
    assert_eq!(n.y, 20);
    assert_eq!(n.width, 5);
    assert_eq!(n.height, 3);
    assert_eq!(n.label, "");
    assert_eq!(n.shape, NodeShape::Rectangle);
}

#[test]
fn test_routed_edge_new() {
    let e = RoutedEdge::new("A", "B", EdgeType::Arrow);
    assert_eq!(e.from_id, "A");
    assert_eq!(e.to_id, "B");
    assert!(e.label.is_none());
    assert!(e.waypoints.is_empty());
}

#[test]
fn test_layout_result_new() {
    let lr = LayoutResult::new(Direction::TD);
    assert_eq!(lr.direction, Direction::TD);
    assert!(lr.nodes.is_empty());
    assert!(lr.edges.is_empty());
    assert!(lr.subgraph_members.is_empty());
    assert!(lr.subgraph_descriptions.is_empty());
}

#[test]
fn test_dummy_prefix() {
    assert_eq!(DUMMY_PREFIX, "__dummy_");
}

#[test]
fn test_compound_prefix() {
    assert_eq!(COMPOUND_PREFIX, "__sg_");
}
