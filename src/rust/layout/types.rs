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

#[cfg(test)]
#[path = "../../../tests/rust/test_layout_types.rs"]
mod tests;
