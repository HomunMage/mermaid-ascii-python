//! Layout engine — convenience API for full graph layout.
//!
//! Mirrors Python's layout/engine.py.

pub mod graph;
pub mod sugiyama;
pub mod types;

pub use graph::GraphIR;
pub use types::{LayoutNode, LayoutResult, Point, RoutedEdge};

use crate::config::RenderConfig;
use sugiyama::SugiyamaLayout;

/// Run the full layout pipeline with default padding.
pub fn full_layout(gir: &GraphIR) -> LayoutResult {
    SugiyamaLayout::layout(gir, sugiyama::NODE_PADDING)
}

/// Run the full layout pipeline with a custom config.
pub fn full_layout_with_config(gir: &GraphIR, config: &RenderConfig) -> LayoutResult {
    SugiyamaLayout::layout(gir, config.padding as i64)
}
