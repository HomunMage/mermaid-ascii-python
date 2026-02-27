//! test_graph.rs — Integration tests for dep/graph.rs
//!
//! Exercises all 14 public functions of the petgraph wrapper:
//!   graph_new, graph_add_node, graph_add_edge, graph_ensure_node,
//!   graph_successors, graph_predecessors, graph_in_degree, graph_out_degree,
//!   graph_nodes, graph_edges, graph_node_count, graph_edge_count,
//!   graph_is_dag, graph_topo_sort, graph_copy.

use mermaid_hom::graph::*;

// ── Construction ──────────────────────────────────────────────────────────────

#[test]
fn test_new_graph_is_empty() {
    let g = graph_new();
    assert_eq!(graph_node_count(&g), 0);
    assert_eq!(graph_edge_count(&g), 0);
}

// ── graph_add_node ────────────────────────────────────────────────────────────

#[test]
fn test_add_node_basic() {
    let mut g = graph_new();
    graph_add_node(&mut g, "n1", "Node One", "Rectangle", None);
    assert_eq!(graph_node_count(&g), 1);
    assert_eq!(graph_nodes(&g), vec!["n1"]);
}

#[test]
fn test_add_node_with_subgraph() {
    let mut g = graph_new();
    graph_add_node(&mut g, "n1", "N1", "Diamond", Some("sg_a"));
    let idx = g.node_index["n1"];
    assert_eq!(g.digraph[idx].subgraph, Some("sg_a".to_string()));
}

#[test]
fn test_add_node_dedup_keeps_first() {
    let mut g = graph_new();
    graph_add_node(&mut g, "x", "first", "Rectangle", None);
    graph_add_node(&mut g, "x", "second", "Rounded", None);
    assert_eq!(graph_node_count(&g), 1);
    let idx = g.node_index["x"];
    assert_eq!(g.digraph[idx].label, "first");
}

// ── graph_add_edge ────────────────────────────────────────────────────────────

#[test]
fn test_add_edge_creates_implicit_nodes() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    assert_eq!(graph_node_count(&g), 2);
    assert_eq!(graph_edge_count(&g), 1);
}

#[test]
fn test_add_edge_with_label() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "A", "B", "Arrow", Some("yes"));
    let eidx = g.digraph.edge_indices().next().unwrap();
    assert_eq!(g.digraph[eidx].label, Some("yes".to_string()));
    assert_eq!(g.digraph[eidx].edge_type, "Arrow");
}

// ── graph_successors / graph_predecessors ─────────────────────────────────────

#[test]
fn test_successors_sorted() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "A", "C", "Arrow", None);
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    assert_eq!(graph_successors(&g, "A"), vec!["B", "C"]);
}

#[test]
fn test_predecessors_sorted() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "C", "B", "Arrow", None);
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    assert_eq!(graph_predecessors(&g, "B"), vec!["A", "C"]);
}

#[test]
fn test_successors_missing_node_empty() {
    let g = graph_new();
    assert!(graph_successors(&g, "ghost").is_empty());
    assert!(graph_predecessors(&g, "ghost").is_empty());
}

// ── graph_in_degree / graph_out_degree ────────────────────────────────────────

#[test]
fn test_degrees() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    graph_add_edge(&mut g, "A", "C", "Arrow", None);
    graph_add_edge(&mut g, "D", "B", "Arrow", None);

    assert_eq!(graph_out_degree(&g, "A"), 2);
    assert_eq!(graph_in_degree(&g, "B"), 2);
    assert_eq!(graph_in_degree(&g, "A"), 0);
    assert_eq!(graph_out_degree(&g, "B"), 0);
    assert_eq!(graph_in_degree(&g, "missing"), 0);
    assert_eq!(graph_out_degree(&g, "missing"), 0);
}

// ── graph_nodes / graph_edges ─────────────────────────────────────────────────

#[test]
fn test_nodes_sorted_alpha() {
    let mut g = graph_new();
    graph_add_node(&mut g, "z", "Z", "Rectangle", None);
    graph_add_node(&mut g, "a", "A", "Rectangle", None);
    graph_add_node(&mut g, "m", "M", "Rectangle", None);
    assert_eq!(graph_nodes(&g), vec!["a", "m", "z"]);
}

#[test]
fn test_edges_sorted() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "B", "C", "Line", None);
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    let edges = graph_edges(&g);
    assert_eq!(
        edges,
        vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ]
    );
}

// ── graph_is_dag / graph_topo_sort ───────────────────────────────────────────

#[test]
fn test_dag_chain() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    graph_add_edge(&mut g, "B", "C", "Arrow", None);
    assert!(graph_is_dag(&g));
    let order = graph_topo_sort(&g).unwrap();
    let pos = |id: &str| order.iter().position(|x| x == id).unwrap();
    assert!(pos("A") < pos("B") && pos("B") < pos("C"));
}

#[test]
fn test_not_dag_with_self_loop() {
    let mut g = graph_new();
    graph_add_node(&mut g, "A", "A", "Rectangle", None);
    graph_add_edge(&mut g, "A", "A", "Arrow", None);
    assert!(!graph_is_dag(&g));
    assert!(graph_topo_sort(&g).is_none());
}

#[test]
fn test_not_dag_with_cycle() {
    let mut g = graph_new();
    graph_add_edge(&mut g, "A", "B", "Arrow", None);
    graph_add_edge(&mut g, "B", "C", "Arrow", None);
    graph_add_edge(&mut g, "C", "A", "Arrow", None);
    assert!(!graph_is_dag(&g));
    assert!(graph_topo_sort(&g).is_none());
}

// ── graph_copy ────────────────────────────────────────────────────────────────

#[test]
fn test_copy_independence() {
    let mut g = graph_new();
    graph_add_node(&mut g, "X", "X", "Diamond", None);
    graph_add_edge(&mut g, "X", "Y", "Arrow", None);

    let mut g2 = graph_copy(&g);

    // Mutation of original does not affect copy.
    graph_add_node(&mut g, "Z", "Z", "Rectangle", None);
    assert_eq!(graph_node_count(&g2), 2); // only X and Y

    // Mutation of copy does not affect original.
    graph_add_node(&mut g2, "W", "W", "Rounded", None);
    assert_eq!(graph_node_count(&g), 3); // X, Y, Z
}
