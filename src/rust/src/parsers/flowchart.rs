//! Recursive descent parser for Mermaid flowchart/graph syntax.
//!
//! Mirrors Python's parsers/flowchart.py.

use crate::syntax::types::Graph;

use super::base::Parser;

/// Recursive descent parser for Mermaid flowchart/graph diagrams.
pub struct FlowchartParser;

impl Parser for FlowchartParser {
    fn parse(&self, _src: &str) -> Result<Graph, String> {
        // TODO: implement in Phase 2
        Err("flowchart parser not yet implemented".to_string())
    }
}
