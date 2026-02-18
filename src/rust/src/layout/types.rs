//! Layout types: LayoutNode, RoutedEdge, Point, LayoutResult.
//!
//! Mirrors Python's layout/types.py.

use crate::syntax::types::{Direction, EdgeType, NodeShape};

// ─── Constants ────────────────────────────────────────────────────────────────

pub const DUMMY_PREFIX: &str = "__dummy_";
pub const COMPOUND_PREFIX: &str = "__sg_";

// ─── Point ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Point {
    pub x: i64,
    pub y: i64,
}

impl Point {
    pub fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

// ─── LayoutNode ───────────────────────────────────────────────────────────────

/// A node with computed layout position and dimensions.
#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub id: String,
    pub layer: usize,
    pub order: usize,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub label: String,
    pub shape: NodeShape,
}

impl LayoutNode {
    pub fn new(
        id: impl Into<String>,
        layer: usize,
        order: usize,
        x: i64,
        y: i64,
        width: i64,
        height: i64,
    ) -> Self {
        Self {
            id: id.into(),
            layer,
            order,
            x,
            y,
            width,
            height,
            label: String::new(),
            shape: NodeShape::Rectangle,
        }
    }
}

// ─── RoutedEdge ───────────────────────────────────────────────────────────────

/// An edge with computed orthogonal waypoints.
#[derive(Debug, Clone)]
pub struct RoutedEdge {
    pub from_id: String,
    pub to_id: String,
    pub label: Option<String>,
    pub edge_type: EdgeType,
    pub waypoints: Vec<Point>,
}

impl RoutedEdge {
    pub fn new(from_id: impl Into<String>, to_id: impl Into<String>, edge_type: EdgeType) -> Self {
        Self {
            from_id: from_id.into(),
            to_id: to_id.into(),
            label: None,
            edge_type,
            waypoints: Vec::new(),
        }
    }
}

// ─── LayoutResult ─────────────────────────────────────────────────────────────

/// The full output of the layout pipeline.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<RoutedEdge>,
    pub direction: Direction,
    /// (subgraph_name, [member_node_ids])
    pub subgraph_members: Vec<(String, Vec<String>)>,
    /// subgraph_name → description
    pub subgraph_descriptions: std::collections::HashMap<String, String>,
}

impl LayoutResult {
    pub fn new(direction: Direction) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            direction,
            subgraph_members: Vec::new(),
            subgraph_descriptions: std::collections::HashMap::new(),
        }
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
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
}
