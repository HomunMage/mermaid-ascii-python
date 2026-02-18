/// AST data structures for Mermaid flowchart syntax.
///
/// These types represent the parsed form of the input DSL:
/// enums (Direction, NodeShape, EdgeType) and structs (Graph, Node, Edge, Subgraph, Attr).
/// Mirrors Python's syntax/types.py 1:1.

// ─── Direction ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Direction {
    LR,
    RL,
    #[default]
    TD,
    BT,
}

// ─── NodeShape ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NodeShape {
    #[default]
    Rectangle, // id[Label]
    Rounded, // id(Label)
    Diamond, // id{Label}
    Circle,  // id((Label))
}

// ─── EdgeType ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeType {
    Arrow,       // -->
    Line,        // ---
    DottedArrow, // -.->
    DottedLine,  // -.-
    ThickArrow,  // ==>
    ThickLine,   // ===
    BidirArrow,  // <-->
    BidirDotted, // <-.->
    BidirThick,  // <==>
}

// ─── Attr ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub key: String,
    pub value: String,
}

// ─── Node ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Mermaid identifier (e.g. "A", "Start", "my-node").
    pub id: String,
    /// Display label (e.g. "Hello World"). Defaults to id if no shape bracket.
    pub label: String,
    pub shape: NodeShape,
    pub attrs: Vec<Attr>,
}

impl Node {
    pub fn new(id: impl Into<String>, label: impl Into<String>, shape: NodeShape) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shape,
            attrs: Vec::new(),
        }
    }

    /// Create a bare node (id = label, default Rectangle shape).
    pub fn bare(id: impl Into<String>) -> Self {
        let id = id.into();
        let label = id.clone();
        Self {
            id,
            label,
            shape: NodeShape::Rectangle,
            attrs: Vec::new(),
        }
    }
}

// ─── Edge ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edge {
    /// ID of the source node.
    pub from_id: String,
    /// ID of the target node.
    pub to_id: String,
    pub edge_type: EdgeType,
    /// Optional inline label on the edge (from |text| syntax).
    pub label: Option<String>,
    pub attrs: Vec<Attr>,
}

impl Edge {
    pub fn new(from_id: impl Into<String>, to_id: impl Into<String>, edge_type: EdgeType) -> Self {
        Self {
            from_id: from_id.into(),
            to_id: to_id.into(),
            edge_type,
            label: None,
            attrs: Vec::new(),
        }
    }
}

// ─── Subgraph ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subgraph {
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    /// Nested subgraphs.
    pub subgraphs: Vec<Subgraph>,
    /// Optional description text shown inside the subgraph box.
    pub description: Option<String>,
    /// Optional direction override within this subgraph.
    pub direction: Option<Direction>,
}

impl Subgraph {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            subgraphs: Vec::new(),
            description: None,
            direction: None,
        }
    }
}

// ─── Graph (top-level AST) ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Graph {
    pub direction: Direction,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub subgraphs: Vec<Subgraph>,
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_default() {
        assert_eq!(Direction::default(), Direction::TD);
    }

    #[test]
    fn test_node_shape_default() {
        assert_eq!(NodeShape::default(), NodeShape::Rectangle);
    }

    #[test]
    fn test_node_new() {
        let n = Node::new("A", "Hello", NodeShape::Rounded);
        assert_eq!(n.id, "A");
        assert_eq!(n.label, "Hello");
        assert_eq!(n.shape, NodeShape::Rounded);
        assert!(n.attrs.is_empty());
    }

    #[test]
    fn test_node_bare() {
        let n = Node::bare("B");
        assert_eq!(n.id, "B");
        assert_eq!(n.label, "B");
        assert_eq!(n.shape, NodeShape::Rectangle);
    }

    #[test]
    fn test_edge_new() {
        let e = Edge::new("A", "B", EdgeType::Arrow);
        assert_eq!(e.from_id, "A");
        assert_eq!(e.to_id, "B");
        assert_eq!(e.edge_type, EdgeType::Arrow);
        assert!(e.label.is_none());
    }

    #[test]
    fn test_subgraph_new() {
        let sg = Subgraph::new("Group");
        assert_eq!(sg.name, "Group");
        assert!(sg.nodes.is_empty());
        assert!(sg.edges.is_empty());
        assert!(sg.direction.is_none());
    }

    #[test]
    fn test_graph_new() {
        let g = Graph::new();
        assert_eq!(g.direction, Direction::TD);
        assert!(g.nodes.is_empty());
        assert!(g.edges.is_empty());
        assert!(g.subgraphs.is_empty());
    }

    #[test]
    fn test_edge_with_label() {
        let mut e = Edge::new("X", "Y", EdgeType::DottedArrow);
        e.label = Some("my label".to_string());
        assert_eq!(e.label.as_deref(), Some("my label"));
    }
}
