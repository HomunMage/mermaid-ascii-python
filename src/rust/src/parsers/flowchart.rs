//! Recursive descent parser for Mermaid flowchart/graph syntax.
//!
//! Mirrors Python's parsers/flowchart.py.

use crate::syntax::types::Graph;

use super::base::{Cursor, Parser};

/// Recursive descent parser for Mermaid flowchart/graph diagrams.
pub struct FlowchartParser;

impl Parser for FlowchartParser {
    fn parse(&self, src: &str) -> Result<Graph, String> {
        let mut cursor = Cursor::new(src);
        Ok(cursor.parse_graph())
    }
}
