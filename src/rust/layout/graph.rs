//! GraphIR — converts AST into a petgraph DiGraph for layout and analysis.
//!
//! Mirrors Python's layout/graph.py 1:1.
//! Flattens subgraphs into the main node/edge lists while preserving
//! subgraph membership for later rendering.

use std::collections::{HashMap, HashSet};

use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::syntax::types::{
    Attr, Direction, EdgeType, Graph as AstGraph, Node as AstNode, NodeShape,
    Subgraph as AstSubgraph,
};

/// Node data stored in the petgraph DiGraph.
#[derive(Debug, Clone)]
pub struct NodeData {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub attrs: Vec<Attr>,
    /// Subgraph name this node belongs to, if any.
    pub subgraph: Option<String>,
}

/// Edge data stored in the petgraph DiGraph.
#[derive(Debug, Clone)]
pub struct EdgeData {
    pub edge_type: EdgeType,
    pub label: Option<String>,
    pub attrs: Vec<Attr>,
}

/// Graph intermediate representation.
///
/// Wraps petgraph DiGraph and adds Mermaid-specific metadata.
pub struct GraphIR {
    pub digraph: DiGraph<NodeData, EdgeData>,
    pub direction: Direction,
    /// Maps node id → petgraph NodeIndex.
    pub node_index: HashMap<String, NodeIndex>,
    /// List of (subgraph_name, member_node_ids) in order of encounter.
    pub subgraph_members: Vec<(String, Vec<String>)>,
    /// Maps subgraph name → description text.
    pub subgraph_descriptions: HashMap<String, String>,
}

impl GraphIR {
    /// Build a GraphIR from the parsed AST.
    pub fn from_ast(ast: &AstGraph) -> Self {
        let mut digraph: DiGraph<NodeData, EdgeData> = DiGraph::new();
        let mut node_index: HashMap<String, NodeIndex> = HashMap::new();
        let mut subgraph_members: Vec<(String, Vec<String>)> = Vec::new();

        // Collect all subgraph names to skip top-level nodes that shadow them.
        let sg_names: HashSet<String> = ast.subgraphs.iter().map(|sg| sg.name.clone()).collect();

        // Add top-level nodes (skip any whose id matches a subgraph name).
        for node in &ast.nodes {
            if !sg_names.contains(&node.id) {
                add_node_if_absent(&mut digraph, &mut node_index, node, None);
            }
        }

        // Collect subgraph members (adds nodes with their subgraph membership).
        for sg in &ast.subgraphs {
            collect_subgraph(sg, &mut digraph, &mut node_index, &mut subgraph_members);
        }

        // Add top-level edges (ensures endpoints exist as placeholder nodes).
        for edge in &ast.edges {
            ensure_node(&mut digraph, &mut node_index, &edge.from_id);
            ensure_node(&mut digraph, &mut node_index, &edge.to_id);
            add_edge(&mut digraph, &node_index, edge);
        }

        // Add edges from inside subgraphs.
        for sg in &ast.subgraphs {
            collect_subgraph_edges(sg, &mut digraph, &mut node_index);
        }

        // Collect subgraph descriptions.
        let mut subgraph_descriptions: HashMap<String, String> = HashMap::new();
        for sg in &ast.subgraphs {
            collect_descriptions(sg, &mut subgraph_descriptions);
        }

        Self {
            digraph,
            direction: ast.direction.clone(),
            node_index,
            subgraph_members,
            subgraph_descriptions,
        }
    }

    pub fn node_count(&self) -> usize {
        self.digraph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.digraph.edge_count()
    }

    /// Returns true if the graph is a directed acyclic graph (no cycles).
    pub fn is_dag(&self) -> bool {
        !is_cyclic_directed(&self.digraph)
    }

    /// Returns topological order of node ids, or None if the graph has cycles.
    pub fn topological_order(&self) -> Option<Vec<String>> {
        match toposort(&self.digraph, None) {
            Ok(indices) => {
                let ids = indices
                    .into_iter()
                    .map(|idx| self.digraph[idx].id.clone())
                    .collect();
                Some(ids)
            }
            Err(_) => None,
        }
    }

    pub fn in_degree(&self, id: &str) -> usize {
        match self.node_index.get(id) {
            None => 0,
            Some(&idx) => self
                .digraph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .count(),
        }
    }

    pub fn out_degree(&self, id: &str) -> usize {
        match self.node_index.get(id) {
            None => 0,
            Some(&idx) => self
                .digraph
                .edges_directed(idx, petgraph::Direction::Outgoing)
                .count(),
        }
    }

    /// Returns sorted adjacency list: Vec<(node_id, sorted_successor_ids)>.
    pub fn adjacency_list(&self) -> Vec<(String, Vec<String>)> {
        let mut result: Vec<(String, Vec<String>)> = self
            .digraph
            .node_indices()
            .map(|idx| {
                let id = self.digraph[idx].id.clone();
                let mut neighbors: Vec<String> = self
                    .digraph
                    .neighbors(idx)
                    .map(|n| self.digraph[n].id.clone())
                    .collect();
                neighbors.sort();
                (id, neighbors)
            })
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn add_node_if_absent(
    digraph: &mut DiGraph<NodeData, EdgeData>,
    node_index: &mut HashMap<String, NodeIndex>,
    ast_node: &AstNode,
    subgraph_name: Option<String>,
) {
    if !node_index.contains_key(&ast_node.id) {
        let data = NodeData {
            id: ast_node.id.clone(),
            label: ast_node.label.clone(),
            shape: ast_node.shape.clone(),
            attrs: ast_node.attrs.clone(),
            subgraph: subgraph_name,
        };
        let idx = digraph.add_node(data);
        node_index.insert(ast_node.id.clone(), idx);
    }
}

fn ensure_node(
    digraph: &mut DiGraph<NodeData, EdgeData>,
    node_index: &mut HashMap<String, NodeIndex>,
    node_id: &str,
) {
    if !node_index.contains_key(node_id) {
        let data = NodeData {
            id: node_id.to_string(),
            label: node_id.to_string(),
            shape: NodeShape::Rectangle,
            attrs: Vec::new(),
            subgraph: None,
        };
        let idx = digraph.add_node(data);
        node_index.insert(node_id.to_string(), idx);
    }
}

fn add_edge(
    digraph: &mut DiGraph<NodeData, EdgeData>,
    node_index: &HashMap<String, NodeIndex>,
    edge: &crate::syntax::types::Edge,
) {
    let from_idx = node_index[&edge.from_id];
    let to_idx = node_index[&edge.to_id];
    let data = EdgeData {
        edge_type: edge.edge_type.clone(),
        label: edge.label.clone(),
        attrs: edge.attrs.clone(),
    };
    digraph.add_edge(from_idx, to_idx, data);
}

fn collect_subgraph(
    sg: &AstSubgraph,
    digraph: &mut DiGraph<NodeData, EdgeData>,
    node_index: &mut HashMap<String, NodeIndex>,
    subgraph_members: &mut Vec<(String, Vec<String>)>,
) {
    let mut member_ids: Vec<String> = Vec::new();
    for node in &sg.nodes {
        add_node_if_absent(digraph, node_index, node, Some(sg.name.clone()));
        member_ids.push(node.id.clone());
    }
    subgraph_members.push((sg.name.clone(), member_ids));
    for nested in &sg.subgraphs {
        collect_subgraph(nested, digraph, node_index, subgraph_members);
    }
}

fn collect_subgraph_edges(
    sg: &AstSubgraph,
    digraph: &mut DiGraph<NodeData, EdgeData>,
    node_index: &mut HashMap<String, NodeIndex>,
) {
    for edge in &sg.edges {
        ensure_node(digraph, node_index, &edge.from_id);
        ensure_node(digraph, node_index, &edge.to_id);
        add_edge(digraph, node_index, edge);
    }
    for nested in &sg.subgraphs {
        collect_subgraph_edges(nested, digraph, node_index);
    }
}

fn collect_descriptions(sg: &AstSubgraph, descriptions: &mut HashMap<String, String>) {
    if let Some(desc) = &sg.description {
        descriptions.insert(sg.name.clone(), desc.clone());
    }
    for nested in &sg.subgraphs {
        collect_descriptions(nested, descriptions);
    }
}

#[cfg(test)]
#[path = "../../../tests/rust/test_layout_graph.rs"]
mod tests;
