//! SVG renderer — converts layout IR to an SVG string.
//!
//! Ported from legacy `src/rust/renderers/svg.rs`, adapted for the current
//! `graph::NodeLayoutList` / `graph::EdgeRouteList` accessor API.
//!
//! Call `render()` after running the full layout pipeline (Phases 1–6).
//! For LR/RL direction the caller must transpose node/edge coordinates
//! *before* calling `render()` (same as the ASCII renderer).

use crate::graph;

// ── Constants ────────────────────────────────────────────────────────────────

const CELL_W: i32 = 10;
const CELL_H: i32 = 20;
const FONT_SIZE: i32 = 14;
const FONT_FAMILY: &str = "monospace";
const PADDING: i32 = 20;

/// Node-ID prefix used by the layout algorithm for dummy/intermediate nodes.
const DUMMY_PREFIX: &str = "__dummy_";

const FILL_STROKE: &str = r#"fill="white" stroke="black" stroke-width="1.5""#;
const SG_STROKE: &str = r##"fill="none" stroke="#888" stroke-width="1" stroke-dasharray="4 2""##;

// ── Helpers ──────────────────────────────────────────────────────────────────

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn font(size: i32) -> String {
    format!(r#"font-family="{FONT_FAMILY}" font-size="{size}""#)
}

/// Convert a column index to a pixel x-coordinate.
fn px(col: i32) -> i32 {
    PADDING + col * CELL_W
}

/// Convert a row index to a pixel y-coordinate.
fn py(row: i32) -> i32 {
    PADDING + row * CELL_H
}

// ── Edge stroke / marker helpers ─────────────────────────────────────────────

fn stroke_style(et: &str) -> &'static str {
    match et {
        "DottedArrow" | "DottedLine" | "BidirDotted" => r#"stroke-dasharray="6 4""#,
        "ThickArrow" | "ThickLine" | "BidirThick" => r#"stroke-width="3""#,
        _ => "",
    }
}

fn is_arrow(et: &str) -> bool {
    matches!(
        et,
        "Arrow" | "DottedArrow" | "ThickArrow" | "BidirArrow" | "BidirDotted" | "BidirThick"
    )
}

fn is_bidir(et: &str) -> bool {
    matches!(et, "BidirArrow" | "BidirDotted" | "BidirThick")
}

// ── Shape rendering ───────────────────────────────────────────────────────────

fn render_node(x: i32, y: i32, w: i32, h: i32, label: &str, shape: &str) -> String {
    let sx = px(x);
    let sy = py(y);
    let sw = w * CELL_W;
    let sh = h * CELL_H;
    let cx = sx + sw / 2;
    let cy = sy + sh / 2;
    let label_esc = escape(label);
    let lines: Vec<&str> = label_esc.split('\n').collect();
    let f = font(FONT_SIZE);

    let label_svg = if lines.len() == 1 {
        format!(
            r#"<text x="{cx}" y="{cy}" dominant-baseline="central" text-anchor="middle" {f}>{}</text>"#,
            lines[0]
        )
    } else {
        let total_h = lines.len() as i32 * (FONT_SIZE + 2);
        let start_y = cy - total_h / 2 + FONT_SIZE / 2;
        let tspans: String = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                let ty = start_y + i as i32 * (FONT_SIZE + 2);
                format!(r#"<tspan x="{cx}" y="{ty}">{line}</tspan>"#)
            })
            .collect();
        format!(r#"<text text-anchor="middle" {f}>{tspans}</text>"#)
    };

    let shape_svg = match shape {
        "Rounded" => {
            let r = sw.min(sh) / 4;
            format!(
                r#"<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" rx="{r}" {FILL_STROKE}/>"#
            )
        }
        "Diamond" => {
            let pts = format!("{cx},{sy} {},{cy} {cx},{} {sx},{cy}", sx + sw, sy + sh);
            format!(r#"<polygon points="{pts}" {FILL_STROKE}/>"#)
        }
        "Circle" => {
            let rx = sw / 2;
            let ry = sh / 2;
            format!(r#"<ellipse cx="{cx}" cy="{cy}" rx="{rx}" ry="{ry}" {FILL_STROKE}/>"#)
        }
        _ => {
            // Rectangle (default)
            format!(r#"<rect x="{sx}" y="{sy}" width="{sw}" height="{sh}" rx="0" {FILL_STROKE}/>"#)
        }
    };

    format!("{shape_svg}\n{label_svg}")
}

// ── Edge rendering ────────────────────────────────────────────────────────────

fn render_edge(waypoints: &[(i32, i32)], edge_type: &str, label: &str) -> String {
    if waypoints.len() < 2 {
        return String::new();
    }

    let style = stroke_style(edge_type);
    let mut markers = String::new();
    if is_arrow(edge_type) {
        markers.push_str(r#" marker-end="url(#arrowhead)""#);
    }
    if is_bidir(edge_type) {
        markers.push_str(r#" marker-start="url(#arrowhead-rev)""#);
    }

    let pts: String = waypoints
        .iter()
        .map(|(x, y)| format!("{},{}", px(*x), py(*y)))
        .collect::<Vec<_>>()
        .join(" ");

    let mut parts = vec![format!(
        r#"<polyline points="{pts}" fill="none" stroke="black" stroke-width="1.5" {style}{markers}/>"#
    )];

    if !label.is_empty() {
        let mid = waypoints.len() / 2;
        let (lx, ly) = waypoints[mid];
        let lsx = px(lx);
        let lsy = py(ly) - 8;
        let f = font(FONT_SIZE - 2);
        parts.push(format!(
            r##"<text x="{lsx}" y="{lsy}" text-anchor="middle" {f} fill="#333">{}</text>"##,
            escape(label)
        ));
    }

    parts.join("\n")
}

// ── Subgraph borders ──────────────────────────────────────────────────────────

/// (from_id, to_id, waypoints, edge_type, label)
type EdgeEntry = (String, String, Vec<(i32, i32)>, String, String);

struct NodePos {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn render_subgraph_borders(
    subgraph_members: &[(String, Vec<String>)],
    node_positions: &std::collections::HashMap<String, NodePos>,
) -> String {
    let mut parts = Vec::new();

    for (sg_name, members) in subgraph_members {
        if members.is_empty() {
            continue;
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for member_id in members {
            if let Some(np) = node_positions.get(member_id.as_str()) {
                let npx = px(np.x);
                let npy = py(np.y);
                let npw = np.width * CELL_W;
                let nph = np.height * CELL_H;
                min_x = min_x.min(npx);
                min_y = min_y.min(npy);
                max_x = max_x.max(npx + npw);
                max_y = max_y.max(npy + nph);
            }
        }

        if min_x == i32::MAX {
            continue;
        }

        let margin = 15i32;
        let bx = min_x - margin;
        let by = min_y - margin;
        let bw = max_x - min_x + 2 * margin;
        let bh = max_y - min_y + 2 * margin;
        let f = font(FONT_SIZE - 2);
        let ty = by + FONT_SIZE + 2;

        parts.push(format!(
            r#"<rect x="{bx}" y="{by}" width="{bw}" height="{bh}" {SG_STROKE}/>"#
        ));
        parts.push(format!(
            r##"<text x="{}" y="{ty}" {f} fill="#666">{}</text>"##,
            bx + 8,
            escape(sg_name)
        ));
    }

    parts.join("\n")
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Render laid-out graph data to an SVG string.
///
/// `nodes` and `edges` must already be transposed for LR/RL direction
/// (call `transpose_layout` first — same contract as the ASCII renderer).
///
/// `direction` controls the SVG `<g>` transform applied for BT/RL:
/// - `"BT"` → `translate(0,H) scale(1,-1)`
/// - `"RL"` → `translate(W,0) scale(-1,1)`
/// - `"LR"` / `"TD"` → no extra transform (layout already correct)
///
/// `subgraph_members` is a slice of `(subgraph_name, [member_node_ids])` pairs
/// used to draw dashed border boxes around each subgraph.
pub fn render(
    nodes: &graph::NodeLayoutList,
    edges: &graph::EdgeRouteList,
    direction: &str,
    subgraph_members: &[(String, Vec<String>)],
) -> String {
    let nn = graph::nll_len(nodes.clone());
    let en = graph::erl_len(edges.clone());

    if nn == 0 {
        return String::new();
    }

    // Build a fast node-id → position map for subgraph border rendering.
    let mut node_positions: std::collections::HashMap<String, NodePos> =
        std::collections::HashMap::new();
    for i in 0..nn {
        let id = graph::nll_get_id(nodes.clone(), i);
        if id.starts_with(DUMMY_PREFIX) {
            continue;
        }
        node_positions.insert(
            id,
            NodePos {
                x: graph::nll_get_x(nodes.clone(), i),
                y: graph::nll_get_y(nodes.clone(), i),
                width: graph::nll_get_width(nodes.clone(), i),
                height: graph::nll_get_height(nodes.clone(), i),
            },
        );
    }

    // Compute canvas size in character-cell units.
    let mut max_col: i32 = 0;
    let mut max_row: i32 = 0;
    for i in 0..nn {
        let id = graph::nll_get_id(nodes.clone(), i);
        if id.starts_with(DUMMY_PREFIX) {
            continue;
        }
        let x = graph::nll_get_x(nodes.clone(), i);
        let y = graph::nll_get_y(nodes.clone(), i);
        let w = graph::nll_get_width(nodes.clone(), i);
        let h = graph::nll_get_height(nodes.clone(), i);
        max_col = max_col.max(x + w + 2);
        max_row = max_row.max(y + h + 2);
    }
    for ei in 0..en {
        let wpc = graph::erl_get_waypoint_count(edges.clone(), ei);
        for wi in 0..wpc {
            let wx = graph::erl_get_waypoint_x(edges.clone(), ei, wi);
            let wy = graph::erl_get_waypoint_y(edges.clone(), ei, wi);
            max_col = max_col.max(wx + 2);
            max_row = max_row.max(wy + 2);
        }
    }

    let svg_w = PADDING * 2 + max_col * CELL_W;
    let svg_h = PADDING * 2 + max_row * CELL_H;

    let transform = match direction {
        "BT" => format!(r#"<g transform="translate(0,{svg_h}) scale(1,-1)">"#),
        "RL" => format!(r#"<g transform="translate({svg_w},0) scale(-1,1)">"#),
        _ => String::new(),
    };

    let mut parts = vec![
        format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{svg_w}" height="{svg_h}" viewBox="0 0 {svg_w} {svg_h}">"#
        ),
        "<defs>".to_string(),
        r#"  <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="10" refY="3.5" orient="auto">"#.to_string(),
        r#"    <polygon points="0 0, 10 3.5, 0 7" fill="black"/>"#.to_string(),
        "  </marker>".to_string(),
        r#"  <marker id="arrowhead-rev" markerWidth="10" markerHeight="7" refX="0" refY="3.5" orient="auto">"#.to_string(),
        r#"    <polygon points="10 0, 0 3.5, 10 7" fill="black"/>"#.to_string(),
        "  </marker>".to_string(),
        "</defs>".to_string(),
        format!(r#"<rect width="{svg_w}" height="{svg_h}" fill="white"/>"#),
    ];

    if !transform.is_empty() {
        parts.push(transform);
    }

    // Subgraph borders (drawn first, behind everything).
    if !subgraph_members.is_empty() {
        let borders = render_subgraph_borders(subgraph_members, &node_positions);
        if !borders.is_empty() {
            parts.push(borders);
        }
    }

    // Edges (behind nodes) — collect and sort for deterministic output.
    let mut edge_data: Vec<EdgeEntry> = Vec::new();
    for ei in 0..en {
        let from_id = graph::erl_get_from(edges.clone(), ei);
        let to_id = graph::erl_get_to(edges.clone(), ei);
        let etype = graph::erl_get_etype(edges.clone(), ei);
        let label = graph::erl_get_label(edges.clone(), ei);
        let wpc = graph::erl_get_waypoint_count(edges.clone(), ei);
        let mut wps = Vec::new();
        for wi in 0..wpc {
            wps.push((
                graph::erl_get_waypoint_x(edges.clone(), ei, wi),
                graph::erl_get_waypoint_y(edges.clone(), ei, wi),
            ));
        }
        edge_data.push((from_id, to_id, wps, etype, label));
    }
    edge_data.sort_by(|a, b| (&a.0, &a.1).cmp(&(&b.0, &b.1)));
    for (_, _, wps, etype, label) in &edge_data {
        let svg = render_edge(wps, etype, label);
        if !svg.is_empty() {
            parts.push(svg);
        }
    }

    // Nodes (on top of edges).
    for i in 0..nn {
        let id = graph::nll_get_id(nodes.clone(), i);
        if id.starts_with(DUMMY_PREFIX) {
            continue;
        }
        let x = graph::nll_get_x(nodes.clone(), i);
        let y = graph::nll_get_y(nodes.clone(), i);
        let w = graph::nll_get_width(nodes.clone(), i);
        let h = graph::nll_get_height(nodes.clone(), i);
        let label = graph::nll_get_label(nodes.clone(), i);
        let shape = graph::nll_get_shape(nodes.clone(), i);
        parts.push(render_node(x, y, w, h, &label, &shape));
    }

    if direction == "BT" || direction == "RL" {
        parts.push("</g>".to_string());
    }

    parts.push("</svg>".to_string());
    parts.join("\n")
}
