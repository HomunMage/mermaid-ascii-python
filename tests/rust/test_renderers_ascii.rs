use super::*;
use crate::layout::types::{LayoutNode, LayoutResult, RoutedEdge};
use crate::syntax::types::{Direction, EdgeType, NodeShape};

fn make_node(id: &str, x: i64, y: i64, w: i64, h: i64) -> LayoutNode {
    let mut n = LayoutNode::new(id, 0, 0, x, y, w, h);
    n.label = id.to_string();
    n.shape = NodeShape::Rectangle;
    n
}

#[test]
fn test_box_chars_rectangle_unicode() {
    let bc = box_chars_for_shape(&NodeShape::Rectangle, CharSet::Unicode);
    assert_eq!(bc.top_left, '┌');
    assert_eq!(bc.top_right, '┐');
}

#[test]
fn test_box_chars_rectangle_ascii() {
    let bc = box_chars_for_shape(&NodeShape::Rectangle, CharSet::Ascii);
    assert_eq!(bc.top_left, '+');
    assert_eq!(bc.horizontal, '-');
}

#[test]
fn test_box_chars_rounded_unicode() {
    let bc = box_chars_for_shape(&NodeShape::Rounded, CharSet::Unicode);
    assert_eq!(bc.top_left, '╭');
    assert_eq!(bc.top_right, '╮');
    assert_eq!(bc.bottom_left, '╰');
    assert_eq!(bc.bottom_right, '╯');
}

#[test]
fn test_box_chars_rounded_ascii() {
    let bc = box_chars_for_shape(&NodeShape::Rounded, CharSet::Ascii);
    assert_eq!(bc.top_left, '+');
}

#[test]
fn test_box_chars_diamond() {
    let bc = box_chars_for_shape(&NodeShape::Diamond, CharSet::Unicode);
    assert_eq!(bc.top_left, '/');
    assert_eq!(bc.top_right, '\\');
    assert_eq!(bc.bottom_left, '\\');
    assert_eq!(bc.bottom_right, '/');
}

#[test]
fn test_box_chars_circle() {
    let bc = box_chars_for_shape(&NodeShape::Circle, CharSet::Unicode);
    assert_eq!(bc.top_left, '(');
    assert_eq!(bc.top_right, ')');
    assert_eq!(bc.vertical, ' ');
}

#[test]
fn test_paint_node_basic() {
    let mut canvas = Canvas::new(20, 5, CharSet::Ascii);
    let ln = make_node("A", 0, 0, 7, 3);
    paint_node(&mut canvas, &ln, &NodeShape::Rectangle, "A");
    assert_eq!(canvas.get(0, 0), '+');
    assert_eq!(canvas.get(6, 0), '+');
    assert_eq!(canvas.get(0, 2), '+');
    assert_eq!(canvas.get(6, 2), '+');
}

#[test]
fn test_render_empty_layout() {
    let renderer = AsciiRenderer::new(true);
    let layout = LayoutResult::new(Direction::TD);
    let result = renderer.render(&layout);
    assert_eq!(result, "");
}

#[test]
fn test_render_single_node() {
    let renderer = AsciiRenderer::new(false);
    let mut layout = LayoutResult::new(Direction::TD);
    layout.nodes.push(make_node("A", 2, 1, 7, 3));
    let result = renderer.render(&layout);
    assert!(!result.is_empty());
    assert!(result.contains('+'));
}

#[test]
fn test_render_edge_arrow() {
    let renderer = AsciiRenderer::new(false);
    let mut layout = LayoutResult::new(Direction::TD);
    layout.nodes.push(make_node("A", 2, 1, 7, 3));
    layout.nodes.push(make_node("B", 2, 8, 7, 3));
    let mut re = RoutedEdge::new("A", "B", EdgeType::Arrow);
    re.waypoints = vec![Point::new(5, 4), Point::new(5, 8)];
    layout.edges.push(re);
    let result = renderer.render(&layout);
    assert!(!result.is_empty());
    // Should have arrow down character
    assert!(result.contains('v') || result.contains('▼'));
}

#[test]
fn test_line_chars_thick() {
    let (h, v) = line_chars_for(&EdgeType::ThickArrow, CharSet::Unicode);
    assert_eq!(h, '═');
    assert_eq!(v, '║');
}

#[test]
fn test_line_chars_dotted() {
    let (h, v) = line_chars_for(&EdgeType::DottedArrow, CharSet::Unicode);
    assert_eq!(h, '╌');
    assert_eq!(v, '╎');
}

#[test]
fn test_line_chars_normal() {
    let (h, v) = line_chars_for(&EdgeType::Arrow, CharSet::Unicode);
    assert_eq!(h, '─');
    assert_eq!(v, '│');
}

#[test]
fn test_remap_char_vertical() {
    assert_eq!(remap_char_vertical('▼'), '▲');
    assert_eq!(remap_char_vertical('▲'), '▼');
    assert_eq!(remap_char_vertical('┌'), '└');
    assert_eq!(remap_char_vertical('X'), 'X');
}

#[test]
fn test_remap_char_horizontal() {
    assert_eq!(remap_char_horizontal('►'), '◄');
    assert_eq!(remap_char_horizontal('◄'), '►');
    assert_eq!(remap_char_horizontal('┌'), '┐');
    assert_eq!(remap_char_horizontal('X'), 'X');
}

#[test]
fn test_flip_vertical() {
    let s = "ABC\nDEF\n";
    let flipped = flip_vertical(s);
    assert!(flipped.starts_with("DEF"));
}

#[test]
fn test_flip_horizontal() {
    let s = "ABC\nDE\n";
    let flipped = flip_horizontal(s);
    let lines: Vec<&str> = flipped.lines().collect();
    assert_eq!(lines[0], "CBA");
}

#[test]
fn test_canvas_dimensions_empty() {
    let (w, h) = canvas_dimensions(&[], &[]);
    assert_eq!(w, 40);
    assert_eq!(h, 10);
}

#[test]
fn test_canvas_dimensions_with_nodes() {
    let n = make_node("A", 0, 0, 10, 5);
    let (w, h) = canvas_dimensions(&[n], &[]);
    assert_eq!(w, 40); // max(40, 0+10+2=12) = 40
    assert_eq!(h, 10); // max(10, 0+5+4=9) = 10
}

#[test]
fn test_canvas_dimensions_large_node() {
    let n = make_node("A", 0, 0, 50, 20);
    let (w, h) = canvas_dimensions(&[n], &[]);
    assert_eq!(w, 52); // 0+50+2
    assert_eq!(h, 24); // 0+20+4
}
