//! Parser registry — detect diagram type and dispatch to the right parser.
//!
//! Mirrors Python's parsers/registry.py.

pub mod base;
pub mod flowchart;

pub use base::Parser;

use crate::syntax::types::Graph;
use flowchart::FlowchartParser;

/// Detect the diagram type from the input source.
///
/// Returns the diagram type as a string (e.g. "flowchart").
pub fn detect_type(src: &str) -> String {
    for line in src.trim().lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }
        let lower = trimmed.to_lowercase();
        if lower.starts_with("flowchart") || lower.starts_with("graph") {
            return "flowchart".to_string();
        }
        break;
    }
    "flowchart".to_string()
}

/// Parse a Mermaid DSL string into a Graph AST.
///
/// Detects the diagram type and dispatches to the appropriate parser.
pub fn parse(src: &str) -> Result<Graph, String> {
    let diagram_type = detect_type(src);
    match diagram_type.as_str() {
        "flowchart" => FlowchartParser.parse(src),
        other => Err(format!("Unsupported diagram type: {other}")),
    }
}
