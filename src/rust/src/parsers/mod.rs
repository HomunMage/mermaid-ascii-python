//! Parser registry — detect diagram type and dispatch to the right parser.
//!
//! Mirrors Python's parsers/registry.py.

pub mod base;
pub mod flowchart;

pub use base::Parser;

use crate::syntax::types::Graph;

/// Detect the diagram type from the input source.
///
/// Returns the diagram type as a string (e.g. "flowchart"), or an error.
pub fn detect_type(_src: &str) -> Result<String, String> {
    // TODO: implement in Phase 2
    Ok("flowchart".to_string())
}

/// Parse a Mermaid DSL string into a Graph AST.
///
/// Detects the diagram type and dispatches to the appropriate parser.
pub fn parse(_src: &str) -> Result<Graph, String> {
    // TODO: implement in Phase 2
    Err("not yet implemented".to_string())
}
