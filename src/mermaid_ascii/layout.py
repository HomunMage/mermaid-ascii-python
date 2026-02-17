"""Layout module — Sugiyama-style graph layout pipeline.

Phases:
  1. Cycle removal  (this file — greedy-FAS approach)
  2. Layer assignment (rank each node)
  3. Crossing minimization (barycenter heuristic)
  4. Coordinate assignment (x/y positions)

1:1 port of layout.rs.
"""

from __future__ import annotations

from dataclasses import dataclass, field

import networkx as nx

# ─── Cycle Removal (Greedy-FAS) ───────────────────────────────────────────────


@dataclass
class CycleRemovalResult:
    """Result of cycle removal: a set of (src, tgt) edge tuples that were
    reversed to make the graph a DAG.

    These are "back-edges" in the original graph. The caller can use this set
    to flip arrow directions in the rendering phase (so the displayed arrow
    still points the "right" way visually).

    In Python/networkx we use (src_id, tgt_id) string tuples as edge identifiers
    since networkx doesn't have stable numeric edge indices like petgraph.
    """

    reversed_edges: set[tuple[str, str]] = field(default_factory=set)


def greedy_fas_ordering(graph: nx.DiGraph) -> list[str]:
    """Compute a node ordering using the greedy-FAS heuristic.

    Returns a list of node ids in an ordering that minimizes back-edges.
    Nodes earlier in the ordering should have outgoing edges going forward.

    Algorithm (Eades, Lin, Smyth 1993):
    - Maintain dynamic in/out degree counters updated as nodes are removed.
    - Repeatedly:
        1. Move all sinks (out_deg == 0) to s2.
        2. Move all sources (in_deg == 0) to s1.
        3. Of remaining nodes in cycles, pick max (out - in) and add to s1.
    - Final ordering: s1 + reversed(s2).
    """
    active: set[str] = set(graph.nodes)

    # Dynamic degree counters (count edges among active nodes only).
    out_deg: dict[str, int] = {}
    in_deg: dict[str, int] = {}
    for node in graph.nodes:
        out_deg[node] = graph.out_degree(node)
        in_deg[node] = graph.in_degree(node)

    # s1: nodes placed at the "left" (sources, high out-degree surplus)
    # s2: nodes placed at the "right" (sinks)
    s1: list[str] = []
    s2: list[str] = []

    while active:
        # Step 1: Pull all sinks (out_deg == 0) into s2.
        changed = True
        while changed:
            changed = False
            sinks = [n for n in active if out_deg[n] == 0]
            if sinks:
                changed = True
                for sink in sinks:
                    active.remove(sink)
                    s2.append(sink)
                    for pred in graph.predecessors(sink):
                        if pred in active:
                            out_deg[pred] -= 1

        # Step 2: Pull all sources (in_deg == 0) into s1.
        changed = True
        while changed:
            changed = False
            sources = [n for n in active if in_deg[n] == 0]
            if sources:
                changed = True
                for source in sources:
                    active.remove(source)
                    s1.append(source)
                    for succ in graph.successors(source):
                        if succ in active:
                            in_deg[succ] -= 1

        # Step 3: If nodes remain (in cycles), pick max (out - in) node.
        if active:
            best = max(active, key=lambda n: out_deg[n] - in_deg[n])
            active.remove(best)
            s1.append(best)
            for succ in graph.successors(best):
                if succ in active:
                    in_deg[succ] -= 1
            for pred in graph.predecessors(best):
                if pred in active:
                    out_deg[pred] -= 1

    # Final ordering: s1 + reversed(s2)
    s2.reverse()
    s1.extend(s2)
    return s1


def remove_cycles(graph: nx.DiGraph) -> tuple[nx.DiGraph, set[tuple[str, str]]]:
    """Remove cycles from a copy of the DiGraph using the greedy-FAS heuristic.

    Returns a tuple of:
    - new_graph: copy of graph with back-edges reversed (self-loops removed)
    - reversed_edges: set of (src_id, tgt_id) tuples that were reversed
      (identified relative to the ORIGINAL graph's edge directions)

    Back-edges are edges where source comes AFTER target in the greedy-FAS
    ordering, or self-loops (which are removed entirely from the DAG).
    """
    if graph.number_of_nodes() == 0:
        return graph.copy(), set()

    # Build node ordering via greedy-FAS.
    ordering = greedy_fas_ordering(graph)

    # Build position map: node_id → position in the ordering.
    position: dict[str, int] = {node: pos for pos, node in enumerate(ordering)}

    # Identify back-edges: edges where source comes AFTER target in ordering,
    # or self-loops (source == target).
    reversed_edges: set[tuple[str, str]] = set()
    for src, tgt in graph.edges():
        is_self_loop = src == tgt
        src_pos = position[src]
        tgt_pos = position[tgt]
        if is_self_loop or src_pos > tgt_pos:
            reversed_edges.add((src, tgt))

    # Build the modified graph with back-edges reversed.
    new_graph: nx.DiGraph = nx.DiGraph()

    # Add all nodes preserving node data.
    for node_id in graph.nodes:
        new_graph.add_node(node_id, **graph.nodes[node_id])

    # Add edges, reversing back-edges. Skip self-loops entirely.
    for src, tgt, edge_attrs in graph.edges(data=True):
        if src == tgt:
            # Self-loop: omit from the DAG entirely.
            continue
        if (src, tgt) in reversed_edges:
            new_graph.add_edge(tgt, src, **edge_attrs)
        else:
            new_graph.add_edge(src, tgt, **edge_attrs)

    return new_graph, reversed_edges
