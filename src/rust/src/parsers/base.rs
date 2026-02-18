//! Parser trait definition.
//!
//! Mirrors Python's parsers/base.py.

use crate::syntax::types::Graph;

/// Trait for diagram parsers.
///
/// Each diagram type (flowchart, sequence, etc.) implements this trait.
pub trait Parser {
    /// Parse the input source string into a Graph AST.
    fn parse(&self, src: &str) -> Result<Graph, String>;
}
