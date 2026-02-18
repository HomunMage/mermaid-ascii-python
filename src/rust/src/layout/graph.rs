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

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::types::{Edge, EdgeType, Graph, Node, NodeShape, Subgraph};

    fn make_graph(
        direction: Direction,
        nodes: Vec<Node>,
        edges: Vec<Edge>,
        subgraphs: Vec<Subgraph>,
    ) -> Graph {
        Graph {
            direction,
            nodes,
            edges,
            subgraphs,
        }
    }

    fn node(id: &str) -> Node {
        Node::bare(id)
    }

    fn node_labeled(id: &str, label: &str, shape: NodeShape) -> Node {
        Node::new(id, label, shape)
    }

    fn edge(from: &str, to: &str) -> Edge {
        Edge::new(from, to, EdgeType::Arrow)
    }

    fn edge_typed(from: &str, to: &str, et: EdgeType) -> Edge {
        Edge::new(from, to, et)
    }

    // ── Basic construction ────────────────────────────────────────────────────

    #[test]
    fn test_empty_graph() {
        let g = make_graph(Direction::TD, vec![], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 0);
        assert_eq!(gir.edge_count(), 0);
    }

    #[test]
    fn test_single_node() {
        let g = make_graph(Direction::TD, vec![node("A")], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 1);
        assert_eq!(gir.edge_count(), 0);
    }

    #[test]
    fn test_direction_preserved() {
        let g = make_graph(Direction::LR, vec![], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.direction, Direction::LR);
    }

    #[test]
    fn test_simple_edge_creates_nodes() {
        let g = make_graph(Direction::TD, vec![], vec![edge("A", "B")], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 2);
        assert_eq!(gir.edge_count(), 1);
    }

    #[test]
    fn test_node_data_stored() {
        let g = make_graph(
            Direction::TD,
            vec![node_labeled("A", "Alpha", NodeShape::Diamond)],
            vec![],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        let idx = gir.node_index["A"];
        let data = &gir.digraph[idx];
        assert_eq!(data.id, "A");
        assert_eq!(data.label, "Alpha");
        assert_eq!(data.shape, NodeShape::Diamond);
        assert!(data.subgraph.is_none());
    }

    #[test]
    fn test_edge_data_stored() {
        let mut e = Edge::new("A", "B", EdgeType::DottedArrow);
        e.label = Some("goes".to_string());
        let g = make_graph(Direction::TD, vec![], vec![e], vec![]);
        let gir = GraphIR::from_ast(&g);
        let from_idx = gir.node_index["A"];
        let to_idx = gir.node_index["B"];
        let edge_idx = gir.digraph.find_edge(from_idx, to_idx).unwrap();
        let data = &gir.digraph[edge_idx];
        assert_eq!(data.edge_type, EdgeType::DottedArrow);
        assert_eq!(data.label.as_deref(), Some("goes"));
    }

    #[test]
    fn test_first_definition_wins() {
        let g = make_graph(
            Direction::TD,
            vec![
                node_labeled("A", "First", NodeShape::Rectangle),
                node_labeled("A", "Second", NodeShape::Diamond),
            ],
            vec![],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 1);
        let idx = gir.node_index["A"];
        assert_eq!(gir.digraph[idx].label, "First");
    }

    // ── Subgraph flattening ───────────────────────────────────────────────────

    #[test]
    fn test_subgraph_members_collected() {
        let mut sg = Subgraph::new("Group");
        sg.nodes = vec![node("X"), node("Y")];
        let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 2);
        assert!(
            gir.subgraph_members
                .contains(&("Group".to_string(), vec!["X".to_string(), "Y".to_string()]))
        );
    }

    #[test]
    fn test_subgraph_node_membership() {
        let mut sg = Subgraph::new("Group");
        sg.nodes = vec![node("X")];
        let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
        let gir = GraphIR::from_ast(&g);
        let idx = gir.node_index["X"];
        assert_eq!(gir.digraph[idx].subgraph.as_deref(), Some("Group"));
    }

    #[test]
    fn test_top_level_node_skipped_if_same_name_as_subgraph() {
        let mut sg = Subgraph::new("Group");
        sg.nodes = vec![node("X")];
        let g = make_graph(Direction::TD, vec![node("Group")], vec![], vec![sg]);
        let gir = GraphIR::from_ast(&g);
        // "X" must be present
        assert!(gir.node_index.contains_key("X"));
    }

    #[test]
    fn test_subgraph_edges_added() {
        let mut sg = Subgraph::new("Group");
        sg.nodes = vec![node("X"), node("Y")];
        sg.edges = vec![edge("X", "Y")];
        let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.edge_count(), 1);
        let x = gir.node_index["X"];
        let y = gir.node_index["Y"];
        assert!(gir.digraph.find_edge(x, y).is_some());
    }

    #[test]
    fn test_subgraph_description() {
        let mut sg = Subgraph::new("Group");
        sg.description = Some("My group".to_string());
        let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(
            gir.subgraph_descriptions.get("Group").map(|s| s.as_str()),
            Some("My group")
        );
    }

    #[test]
    fn test_nested_subgraph() {
        let mut inner = Subgraph::new("Inner");
        inner.nodes = vec![node("Z")];
        let mut outer = Subgraph::new("Outer");
        outer.nodes = vec![node("W")];
        outer.subgraphs = vec![inner];
        let g = make_graph(Direction::TD, vec![], vec![], vec![outer]);
        let gir = GraphIR::from_ast(&g);
        assert!(gir.node_index.contains_key("W"));
        assert!(gir.node_index.contains_key("Z"));
        let z_idx = gir.node_index["Z"];
        assert_eq!(gir.digraph[z_idx].subgraph.as_deref(), Some("Inner"));
    }

    // ── Cycle detection ───────────────────────────────────────────────────────

    #[test]
    fn test_empty_graph_is_dag() {
        let g = make_graph(Direction::TD, vec![], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert!(gir.is_dag());
    }

    #[test]
    fn test_simple_chain_is_dag() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("B", "C")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert!(gir.is_dag());
    }

    #[test]
    fn test_single_cycle_is_not_dag() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("B", "C"), edge("C", "A")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert!(!gir.is_dag());
    }

    #[test]
    fn test_self_loop_is_not_dag() {
        let g = make_graph(Direction::TD, vec![], vec![edge("A", "A")], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert!(!gir.is_dag());
    }

    #[test]
    fn test_two_node_cycle_is_not_dag() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("B", "A")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert!(!gir.is_dag());
    }

    // ── Topological order ─────────────────────────────────────────────────────

    #[test]
    fn test_empty_graph_topo_returns_empty() {
        let g = make_graph(Direction::TD, vec![], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        let result = gir.topological_order();
        assert_eq!(result, Some(vec![]));
    }

    #[test]
    fn test_simple_chain_topo_order() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("B", "C")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        let order = gir.topological_order().unwrap();
        let a_pos = order.iter().position(|x| x == "A").unwrap();
        let b_pos = order.iter().position(|x| x == "B").unwrap();
        let c_pos = order.iter().position(|x| x == "C").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_cycle_topo_returns_none() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("B", "A")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert!(gir.topological_order().is_none());
    }

    #[test]
    fn test_self_loop_topo_returns_none() {
        let g = make_graph(Direction::TD, vec![], vec![edge("A", "A")], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert!(gir.topological_order().is_none());
    }

    // ── Degree queries ────────────────────────────────────────────────────────

    #[test]
    fn test_in_degree_source_node() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("A", "C")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.in_degree("A"), 0);
    }

    #[test]
    fn test_out_degree_source_node() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("A", "C")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.out_degree("A"), 2);
    }

    #[test]
    fn test_in_degree_sink_node() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("C", "B")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.in_degree("B"), 2);
    }

    #[test]
    fn test_out_degree_sink_node() {
        let g = make_graph(Direction::TD, vec![], vec![edge("A", "B")], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.out_degree("B"), 0);
    }

    #[test]
    fn test_degree_unknown_node_returns_zero() {
        let g = make_graph(Direction::TD, vec![node("A")], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.in_degree("NONEXISTENT"), 0);
        assert_eq!(gir.out_degree("NONEXISTENT"), 0);
    }

    #[test]
    fn test_self_loop_degree() {
        let g = make_graph(Direction::TD, vec![], vec![edge("A", "A")], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.in_degree("A"), 1);
        assert_eq!(gir.out_degree("A"), 1);
    }

    // ── Adjacency list ────────────────────────────────────────────────────────

    #[test]
    fn test_empty_adjacency_list() {
        let g = make_graph(Direction::TD, vec![], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.adjacency_list(), vec![]);
    }

    #[test]
    fn test_single_node_adjacency() {
        let g = make_graph(Direction::TD, vec![node("A")], vec![], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.adjacency_list(), vec![("A".to_string(), vec![])]);
    }

    #[test]
    fn test_chain_adjacency() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("B", "C")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        let adj: HashMap<String, Vec<String>> = gir.adjacency_list().into_iter().collect();
        assert_eq!(adj["A"], vec!["B"]);
        assert_eq!(adj["B"], vec!["C"]);
        assert_eq!(adj["C"], Vec::<String>::new());
    }

    #[test]
    fn test_neighbors_sorted() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "C"), edge("A", "B")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        let adj: HashMap<String, Vec<String>> = gir.adjacency_list().into_iter().collect();
        assert_eq!(adj["A"], vec!["B", "C"]);
    }

    #[test]
    fn test_adjacency_list_sorted_by_key() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("C", "A"), edge("B", "A")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        let keys: Vec<String> = gir.adjacency_list().into_iter().map(|(k, _)| k).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
    }

    // ── Edge types ────────────────────────────────────────────────────────────

    #[test]
    fn test_all_edge_types_stored() {
        let types = vec![
            EdgeType::Arrow,
            EdgeType::Line,
            EdgeType::DottedArrow,
            EdgeType::DottedLine,
            EdgeType::ThickArrow,
            EdgeType::ThickLine,
            EdgeType::BidirArrow,
            EdgeType::BidirDotted,
            EdgeType::BidirThick,
        ];
        for et in types {
            let g = make_graph(
                Direction::TD,
                vec![],
                vec![edge_typed("A", "B", et.clone())],
                vec![],
            );
            let gir = GraphIR::from_ast(&g);
            let from_idx = gir.node_index["A"];
            let to_idx = gir.node_index["B"];
            let eidx = gir.digraph.find_edge(from_idx, to_idx).unwrap();
            assert_eq!(gir.digraph[eidx].edge_type, et);
        }
    }

    // ── Node / edge count ─────────────────────────────────────────────────────

    #[test]
    fn test_no_duplicate_nodes_from_shared_edge_endpoint() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![edge("A", "B"), edge("A", "C")],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 3);
    }

    #[test]
    fn test_explicit_and_implicit_same_node() {
        let g = make_graph(Direction::TD, vec![node("A")], vec![edge("A", "B")], vec![]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 2);
    }

    #[test]
    fn test_diamond_graph() {
        let g = make_graph(
            Direction::TD,
            vec![],
            vec![
                edge("A", "B"),
                edge("A", "C"),
                edge("B", "D"),
                edge("C", "D"),
            ],
            vec![],
        );
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 4);
        assert_eq!(gir.edge_count(), 4);
        assert!(gir.is_dag());
    }

    // ── Extended subgraph ─────────────────────────────────────────────────────

    #[test]
    fn test_multiple_subgraphs_members() {
        let mut sg1 = Subgraph::new("SG1");
        sg1.nodes = vec![node("A"), node("B")];
        let mut sg2 = Subgraph::new("SG2");
        sg2.nodes = vec![node("C")];
        let g = make_graph(Direction::TD, vec![], vec![], vec![sg1, sg2]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 3);
        assert_eq!(gir.subgraph_members.len(), 2);
        let names: HashSet<&str> = gir
            .subgraph_members
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        assert!(names.contains("SG1"));
        assert!(names.contains("SG2"));
    }

    #[test]
    fn test_cross_subgraph_edge_at_top_level() {
        let mut sg1 = Subgraph::new("SG1");
        sg1.nodes = vec![node("A")];
        let mut sg2 = Subgraph::new("SG2");
        sg2.nodes = vec![node("B")];
        let g = make_graph(Direction::TD, vec![], vec![edge("A", "B")], vec![sg1, sg2]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.edge_count(), 1);
        let a = gir.node_index["A"];
        let b = gir.node_index["B"];
        assert!(gir.digraph.find_edge(a, b).is_some());
    }

    #[test]
    fn test_deeply_nested_subgraph() {
        let mut innermost = Subgraph::new("Level3");
        innermost.nodes = vec![node("P")];
        let mut middle = Subgraph::new("Level2");
        middle.nodes = vec![node("Q")];
        middle.subgraphs = vec![innermost];
        let mut outer = Subgraph::new("Level1");
        outer.nodes = vec![node("R")];
        outer.subgraphs = vec![middle];
        let g = make_graph(Direction::TD, vec![], vec![], vec![outer]);
        let gir = GraphIR::from_ast(&g);
        assert_eq!(gir.node_count(), 3);
        assert_eq!(gir.subgraph_members.len(), 3);
        let names: HashSet<&str> = gir
            .subgraph_members
            .iter()
            .map(|(n, _)| n.as_str())
            .collect();
        assert!(names.contains("Level1"));
        assert!(names.contains("Level2"));
        assert!(names.contains("Level3"));
    }

    #[test]
    fn test_no_description_when_none() {
        let sg = Subgraph::new("SG");
        let g = make_graph(Direction::TD, vec![], vec![], vec![sg]);
        let gir = GraphIR::from_ast(&g);
        assert!(!gir.subgraph_descriptions.contains_key("SG"));
    }
}
