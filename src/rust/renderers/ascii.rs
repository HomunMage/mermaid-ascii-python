//! ASCII/Unicode renderer for mermaid-ascii.
//!
//! Mirrors Python's renderers/ascii.py.

use std::collections::HashMap;

use super::Renderer;
use super::canvas::{Canvas, Rect};
use super::charset::{Arms, BoxChars, CharSet};
use crate::layout::types::{
    COMPOUND_PREFIX, DUMMY_PREFIX, LayoutNode, LayoutResult, Point, RoutedEdge,
};
use crate::syntax::types::{Direction, EdgeType, NodeShape};

// ─── Node Rendering ───────────────────────────────────────────────────────────

fn box_chars_for_shape(shape: &NodeShape, cs: CharSet) -> BoxChars {
    match shape {
        NodeShape::Rectangle => BoxChars::for_charset(cs),
        NodeShape::Rounded => {
            if cs == CharSet::Ascii {
                return BoxChars::ascii();
            }
            let mut bc = BoxChars::unicode();
            bc.top_left = '╭';
            bc.top_right = '╮';
            bc.bottom_left = '╰';
            bc.bottom_right = '╯';
            bc
        }
        NodeShape::Diamond => {
            let mut bc = BoxChars::for_charset(cs);
            bc.top_left = '/';
            bc.top_right = '\\';
            bc.bottom_left = '\\';
            bc.bottom_right = '/';
            bc
        }
        NodeShape::Circle => {
            let mut bc = BoxChars::for_charset(cs);
            bc.top_left = '(';
            bc.top_right = ')';
            bc.bottom_left = '(';
            bc.bottom_right = ')';
            bc.vertical = ' ';
            bc
        }
    }
}

fn paint_node(canvas: &mut Canvas, ln: &LayoutNode, shape: &NodeShape, label: &str) {
    let bc = box_chars_for_shape(shape, canvas.charset);
    let rect = Rect::new(ln.x, ln.y, ln.width, ln.height);
    canvas.draw_box(rect, &bc);

    let inner_w = (ln.width - 2).max(0) as usize;
    for (i, line) in label.split('\n').enumerate() {
        let label_row = ln.y + 1 + i as i64;
        if label_row < 0 {
            continue;
        }
        let line_len = line.chars().count();
        let pad = inner_w.saturating_sub(line_len) / 2;
        let col_start = ln.x + 1 + pad as i64;
        if col_start >= 0 && label_row >= 0 {
            canvas.write_str(col_start as usize, label_row as usize, line);
        }
    }
}

fn paint_compound_node(
    canvas: &mut Canvas,
    ln: &LayoutNode,
    sg_name: &str,
    description: Option<&str>,
) {
    let bc = BoxChars::for_charset(canvas.charset);
    let rect = Rect::new(ln.x, ln.y, ln.width, ln.height);
    canvas.draw_box(rect, &bc);

    let inner_w = (ln.width - 2).max(0) as usize;
    let title_pad = inner_w.saturating_sub(sg_name.chars().count()) / 2;
    let title_col = ln.x + 1 + title_pad as i64;
    let title_row = ln.y + 1;
    if title_col >= 0 && title_row >= 0 {
        canvas.write_str(title_col as usize, title_row as usize, sg_name);
    }

    if let Some(desc) = description {
        let desc_row = ln.y + ln.height - 2;
        let desc_pad = inner_w.saturating_sub(desc.chars().count()) / 2;
        let desc_col = ln.x + 1 + desc_pad as i64;
        if desc_col >= 0 && desc_row >= 0 {
            canvas.write_str(desc_col as usize, desc_row as usize, desc);
        }
    }
}

fn paint_subgraph_borders(
    subgraph_members: &[(String, Vec<String>)],
    layout_nodes: &[LayoutNode],
    canvas: &mut Canvas,
) {
    let node_pos: HashMap<&str, &LayoutNode> =
        layout_nodes.iter().map(|n| (n.id.as_str(), n)).collect();
    let bc = BoxChars::for_charset(canvas.charset);

    for (sg_name, members) in subgraph_members {
        if members.is_empty() {
            continue;
        }

        let mut min_x = i64::MAX;
        let mut min_y = i64::MAX;
        let mut max_x = i64::MIN;
        let mut max_y = i64::MIN;

        for member_id in members {
            let Some(ln) = node_pos.get(member_id.as_str()) else {
                continue;
            };
            if ln.x < min_x {
                min_x = ln.x;
            }
            if ln.y < min_y {
                min_y = ln.y;
            }
            let right = ln.x + ln.width;
            let bottom = ln.y + ln.height;
            if right > max_x {
                max_x = right;
            }
            if bottom > max_y {
                max_y = bottom;
            }
        }

        if min_x == i64::MAX {
            continue;
        }

        let margin_x: i64 = 2;
        let margin_y: i64 = 1;
        let bx = (min_x - margin_x).max(0);
        let by = (min_y - margin_y).max(0);
        let bw = (max_x + margin_x) - bx;
        let bh = (max_y + margin_y) - by;

        let rect = Rect::new(bx, by, bw, bh);
        canvas.draw_box(rect, &bc);

        let label = format!(" {sg_name} ");
        let label_col = bx + 2;
        if label.len() + 4 <= bw as usize && label_col >= 0 && by >= 0 {
            canvas.write_str(label_col as usize, by as usize, &label);
        }
    }
}

// ─── Edge Rendering ───────────────────────────────────────────────────────────

fn line_chars_for(edge_type: &EdgeType, cs: CharSet) -> (char, char) {
    let bc = BoxChars::for_charset(cs);
    match edge_type {
        EdgeType::ThickArrow | EdgeType::ThickLine | EdgeType::BidirThick => ('═', '║'),
        EdgeType::DottedArrow | EdgeType::DottedLine | EdgeType::BidirDotted => ('╌', '╎'),
        _ => (bc.horizontal, bc.vertical),
    }
}

fn is_arrow_type(edge_type: &EdgeType) -> bool {
    matches!(
        edge_type,
        EdgeType::Arrow
            | EdgeType::DottedArrow
            | EdgeType::ThickArrow
            | EdgeType::BidirArrow
            | EdgeType::BidirDotted
            | EdgeType::BidirThick
    )
}

fn is_bidir_type(edge_type: &EdgeType) -> bool {
    matches!(
        edge_type,
        EdgeType::BidirArrow | EdgeType::BidirDotted | EdgeType::BidirThick
    )
}

fn paint_edge(canvas: &mut Canvas, re: &RoutedEdge, edge_type: &EdgeType) {
    if re.waypoints.len() < 2 {
        return;
    }

    let cs = canvas.charset;
    let (h_ch, v_ch) = line_chars_for(edge_type, cs);
    let bc = BoxChars::for_charset(cs);

    // Draw interior cells of each segment (excluding waypoint endpoints)
    for i in 0..re.waypoints.len() - 1 {
        let p0 = &re.waypoints[i];
        let p1 = &re.waypoints[i + 1];
        if p0.y == p1.y {
            // horizontal segment
            let lo = p0.x.min(p1.x);
            let hi = p0.x.max(p1.x);
            for col in (lo + 1)..hi {
                if col >= 0 && p0.y >= 0 {
                    canvas.set_merge(col as usize, p0.y as usize, h_ch);
                }
            }
        } else if p0.x == p1.x {
            // vertical segment
            let lo = p0.y.min(p1.y);
            let hi = p0.y.max(p1.y);
            for row in (lo + 1)..hi {
                if p0.x >= 0 && row >= 0 {
                    canvas.set_merge(p0.x as usize, row as usize, v_ch);
                }
            }
        }
    }

    // At each waypoint, compute exact arms from incoming/outgoing directions
    for i in 0..re.waypoints.len() {
        let p = &re.waypoints[i];
        let mut arms = Arms::new(false, false, false, false);

        if i > 0 {
            let prev = &re.waypoints[i - 1];
            if prev.x < p.x {
                arms.left = true;
            } else if prev.x > p.x {
                arms.right = true;
            } else if prev.y < p.y {
                arms.up = true;
            } else if prev.y > p.y {
                arms.down = true;
            }
        }

        if i < re.waypoints.len() - 1 {
            let nxt = &re.waypoints[i + 1];
            if nxt.x > p.x {
                arms.right = true;
            } else if nxt.x < p.x {
                arms.left = true;
            } else if nxt.y > p.y {
                arms.down = true;
            } else if nxt.y < p.y {
                arms.up = true;
            }
        }

        // Merge with existing character
        if p.x >= 0 && p.y >= 0 {
            let col = p.x as usize;
            let row = p.y as usize;
            if row < canvas.height && col < canvas.width {
                let existing = canvas.get(col, row);
                if let Some(ea) = Arms::from_char(existing) {
                    let merged = ea.merge(arms);
                    canvas.set(col, row, merged.to_char(cs));
                } else {
                    canvas.set(col, row, arms.to_char(cs));
                }
            }
        }
    }

    // Place arrowheads
    if is_arrow_type(edge_type) {
        let last = &re.waypoints[re.waypoints.len() - 1];
        let prev = &re.waypoints[re.waypoints.len() - 2];
        let arrow = if last.y < prev.y {
            bc.arrow_up
        } else if last.y > prev.y {
            bc.arrow_down
        } else if last.x > prev.x {
            bc.arrow_right
        } else {
            bc.arrow_left
        };
        if last.x >= 0 && last.y >= 0 {
            canvas.set(last.x as usize, last.y as usize, arrow);
        }
    }

    if is_bidir_type(edge_type) {
        let first = &re.waypoints[0];
        let second = &re.waypoints[1];
        let start_arrow = if first.y < second.y {
            bc.arrow_up
        } else if first.y > second.y {
            bc.arrow_down
        } else if first.x > second.x {
            bc.arrow_right
        } else {
            bc.arrow_left
        };
        if first.x >= 0 && first.y >= 0 {
            canvas.set(first.x as usize, first.y as usize, start_arrow);
        }
    }

    if let Some(label) = &re.label {
        let mid = re.waypoints.len() / 2;
        let lp = &re.waypoints[mid];
        let label_y = (lp.y - 1).max(0);
        if lp.x >= 0 && label_y >= 0 {
            canvas.write_str(lp.x as usize, label_y as usize, label);
        }
    }
}

fn paint_exit_stubs(canvas: &mut Canvas, edges: &[RoutedEdge], nodes: &[&LayoutNode]) {
    let node_map: HashMap<&str, &&LayoutNode> = nodes.iter().map(|n| (n.id.as_str(), n)).collect();

    for re in edges {
        if re.waypoints.is_empty() {
            continue;
        }
        let Some(from_node) = node_map.get(re.from_id.as_str()) else {
            continue;
        };

        let first_wp = &re.waypoints[0];
        let nx = from_node.x;
        let ny = from_node.y;
        let nw = from_node.width;
        let nh = from_node.height;
        let center_x = nx + nw / 2;
        let center_y = ny + nh / 2;

        let (stub_x, stub_y, arm_dir): (i64, i64, &str) = if first_wp.y >= ny + nh {
            // Edge exits from bottom border
            (center_x, ny + nh - 1, "down")
        } else if first_wp.y < ny {
            // Edge exits from top border
            (center_x, ny, "up")
        } else if first_wp.x >= nx + nw {
            // Edge exits from right border
            (nx + nw - 1, center_y, "right")
        } else if first_wp.x < nx {
            // Edge exits from left border
            (nx, center_y, "left")
        } else {
            // First waypoint inside node — default to bottom
            (center_x, ny + nh - 1, "down")
        };

        if stub_x < 0 || stub_y < 0 {
            continue;
        }
        let col = stub_x as usize;
        let row = stub_y as usize;
        if row >= canvas.height || col >= canvas.width {
            continue;
        }

        let existing = canvas.get(col, row);
        if let Some(ea) = Arms::from_char(existing) {
            let mut merged = ea;
            match arm_dir {
                "down" => merged.down = true,
                "up" => merged.up = true,
                "right" => merged.right = true,
                "left" => merged.left = true,
                _ => {}
            }
            canvas.set(col, row, merged.to_char(canvas.charset));
        } else {
            let bc = BoxChars::for_charset(canvas.charset);
            let stub_char = match arm_dir {
                "down" => bc.tee_down,
                "up" => bc.tee_up,
                "right" => bc.tee_right,
                "left" => bc.tee_left,
                _ => continue,
            };
            canvas.set(col, row, stub_char);
        }
    }
}

// ─── Direction Transforms ─────────────────────────────────────────────────────

fn transpose_layout(nodes: &mut [LayoutNode], edges: &mut [RoutedEdge]) {
    for n in nodes.iter_mut() {
        std::mem::swap(&mut n.x, &mut n.y);
        std::mem::swap(&mut n.width, &mut n.height);
    }
    for re in edges.iter_mut() {
        for p in re.waypoints.iter_mut() {
            std::mem::swap(&mut p.x, &mut p.y);
        }
    }
}

fn remap_char_vertical(c: char) -> char {
    match c {
        '▼' => '▲',
        '▲' => '▼',
        'v' => '^',
        '^' => 'v',
        '┌' => '└',
        '└' => '┌',
        '┐' => '┘',
        '┘' => '┐',
        '╭' => '╰',
        '╰' => '╭',
        '╮' => '╯',
        '╯' => '╮',
        '┬' => '┴',
        '┴' => '┬',
        other => other,
    }
}

fn remap_char_horizontal(c: char) -> char {
    match c {
        '►' => '◄',
        '◄' => '►',
        '>' => '<',
        '<' => '>',
        '┌' => '┐',
        '┐' => '┌',
        '└' => '┘',
        '┘' => '└',
        '╭' => '╮',
        '╮' => '╭',
        '╰' => '╯',
        '╯' => '╰',
        '├' => '┤',
        '┤' => '├',
        other => other,
    }
}

fn flip_vertical(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let flipped: Vec<String> = lines
        .iter()
        .rev()
        .map(|line| line.chars().map(remap_char_vertical).collect())
        .collect();
    let mut out = flipped.join("\n");
    out.push('\n');
    out
}

fn flip_horizontal(s: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    let flipped: Vec<String> = lines
        .iter()
        .map(|line| {
            let mut chars: Vec<char> = line.chars().collect();
            let pad = max_width - chars.len();
            chars.extend(std::iter::repeat_n(' ', pad));
            chars.reverse();
            let remapped: String = chars.iter().map(|&c| remap_char_horizontal(c)).collect();
            remapped.trim_end().to_string()
        })
        .collect();
    let mut out = flipped.join("\n");
    out.push('\n');
    out
}

// ─── Canvas Sizing ────────────────────────────────────────────────────────────

fn canvas_dimensions(layout_nodes: &[LayoutNode], routed_edges: &[RoutedEdge]) -> (usize, usize) {
    let mut max_col: i64 = 40;
    let mut max_row: i64 = 10;
    for n in layout_nodes {
        if n.id.starts_with(DUMMY_PREFIX) {
            continue;
        }
        max_col = max_col.max(n.x + n.width + 2);
        max_row = max_row.max(n.y + n.height + 4);
    }
    for re in routed_edges {
        for p in &re.waypoints {
            max_col = max_col.max(p.x + 4);
            max_row = max_row.max(p.y + 4);
        }
    }
    (max_col.max(0) as usize, max_row.max(0) as usize)
}

// ─── Public Renderer ──────────────────────────────────────────────────────────

/// ASCII/Unicode text renderer.
///
/// Mirrors Python's AsciiRenderer class.
pub struct AsciiRenderer {
    pub unicode: bool,
}

impl AsciiRenderer {
    pub fn new(unicode: bool) -> Self {
        Self { unicode }
    }
}

impl Renderer for AsciiRenderer {
    fn render(&self, layout: &LayoutResult) -> String {
        let cs = if self.unicode {
            CharSet::Unicode
        } else {
            CharSet::Ascii
        };

        let (mut nodes, mut edges) =
            if layout.direction == Direction::TD || layout.direction == Direction::BT {
                (layout.nodes.clone(), layout.edges.clone())
            } else {
                // LR or RL — transpose so Sugiyama's TD output maps to left-right
                let nodes = layout.nodes.clone();
                let edges = layout
                    .edges
                    .iter()
                    .map(|re| RoutedEdge {
                        from_id: re.from_id.clone(),
                        to_id: re.to_id.clone(),
                        label: re.label.clone(),
                        edge_type: re.edge_type.clone(),
                        waypoints: re.waypoints.iter().map(|p| Point::new(p.x, p.y)).collect(),
                    })
                    .collect();
                (nodes, edges)
            };

        if layout.direction == Direction::LR || layout.direction == Direction::RL {
            transpose_layout(&mut nodes, &mut edges);
        }

        let has_compounds = nodes.iter().any(|n| n.id.starts_with(COMPOUND_PREFIX));
        let real_nodes: Vec<&LayoutNode> = nodes
            .iter()
            .filter(|n| !n.id.starts_with(DUMMY_PREFIX) && !n.id.starts_with(COMPOUND_PREFIX))
            .collect();
        let compound_nodes: Vec<&LayoutNode> = nodes
            .iter()
            .filter(|n| n.id.starts_with(COMPOUND_PREFIX))
            .collect();

        if real_nodes.is_empty() && compound_nodes.is_empty() {
            return String::new();
        }

        let (width, height) = canvas_dimensions(&nodes, &edges);
        let mut canvas = Canvas::new(width, height, cs);

        if has_compounds {
            for ln in &compound_nodes {
                let sg_name = &ln.id[COMPOUND_PREFIX.len()..];
                let desc = layout
                    .subgraph_descriptions
                    .get(sg_name)
                    .map(|s| s.as_str());
                paint_compound_node(&mut canvas, ln, sg_name, desc);
            }
        } else {
            paint_subgraph_borders(&layout.subgraph_members, &nodes, &mut canvas);
        }

        for ln in &real_nodes {
            paint_node(&mut canvas, ln, &ln.shape, &ln.label);
        }

        for re in &edges {
            paint_edge(&mut canvas, re, &re.edge_type);
        }

        // Paint exit stubs on source node borders (┬ at bottom center)
        paint_exit_stubs(&mut canvas, &edges, &real_nodes);

        let rendered = canvas.render_to_string();

        match layout.direction {
            Direction::BT => flip_vertical(&rendered),
            Direction::RL => flip_horizontal(&rendered),
            _ => rendered,
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/rust/test_renderers_ascii.rs"]
mod tests;
