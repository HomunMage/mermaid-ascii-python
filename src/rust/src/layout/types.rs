//! Layout types: LayoutNode, RoutedEdge, Point, LayoutResult.
//!
//! Mirrors Python's layout/types.py.

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
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
    pub layer: usize,
    pub order: usize,
}

// ─── RoutedEdge ───────────────────────────────────────────────────────────────

/// An edge with computed orthogonal waypoints.
#[derive(Debug, Clone)]
pub struct RoutedEdge {
    pub from_id: String,
    pub to_id: String,
    pub waypoints: Vec<Point>,
    pub label: Option<String>,
}

// ─── LayoutResult ─────────────────────────────────────────────────────────────

/// The full output of the layout pipeline.
#[derive(Debug, Clone)]
pub struct LayoutResult {
    pub nodes: Vec<LayoutNode>,
    pub edges: Vec<RoutedEdge>,
}
