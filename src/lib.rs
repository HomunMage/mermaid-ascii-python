//! mermaid-ascii — Mermaid flowchart syntax to ASCII/Unicode text renderer.
//!
//! Implementation in Homun (.hom) source files, compiled to .rs by build.rs.
//!
//! Modules (uncomment as .hom files are completed and wired):
//!   mod types;          // Direction, NodeShape, EdgeType, Node, Edge, Subgraph, Graph
//!   mod config;         // RenderConfig
//!   mod layout_types;   // Point, LayoutNode, RoutedEdge, LayoutResult
//!   mod charset;        // BoxChars, Arms, CharSet
//!   mod canvas;         // Rect, Canvas (2D char grid)
//!   mod parser;         // Cursor tokenizer + flowchart recursive descent
//!   mod pathfinder;     // A* orthogonal edge routing
//!   mod layout;         // Sugiyama 8-phase algorithm
//!   mod render;         // ASCII renderer 7 phases
//!
//! dep/graph.rs — petgraph DiGraph wrapper (pure Rust)

#[path = "dep/graph.rs"]
pub mod graph;

/// Parse a Mermaid flowchart string and render it to ASCII/Unicode art.
///
/// Full implementation wires: parse → GraphIR → layout → renderer.
pub fn render_dsl(
    _src: &str,
    _unicode: bool,
    _padding: usize,
    _direction: Option<&str>,
) -> Result<String, String> {
    Err("not yet implemented — modules are being ported to .hom".to_string())
}
