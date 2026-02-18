//! Layout engine — convenience API for full graph layout.
//!
//! Mirrors Python's layout/engine.py.

pub mod graph;
pub mod sugiyama;
pub mod types;

pub use graph::GraphIR;
pub use types::{LayoutNode, LayoutResult, Point, RoutedEdge};

use crate::config::RenderConfig;

/// Run the full layout pipeline with default padding.
///
/// Returns (layout_nodes, routed_edges).
pub fn full_layout(gir: &GraphIR) -> LayoutResult {
    // TODO: implement in Phase 5
    LayoutResult::new(gir.direction.clone())
}

/// Run the full layout pipeline with a custom config.
///
/// Returns (layout_nodes, routed_edges).
pub fn full_layout_with_config(gir: &GraphIR, _config: &RenderConfig) -> LayoutResult {
    // TODO: implement in Phase 5
    LayoutResult::new(gir.direction.clone())
}
