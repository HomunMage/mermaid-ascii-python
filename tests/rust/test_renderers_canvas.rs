use super::*;

#[test]
fn test_rect_new() {
    let r = Rect::new(1, 2, 10, 5);
    assert_eq!(r.x, 1);
    assert_eq!(r.y, 2);
    assert_eq!(r.width, 10);
    assert_eq!(r.height, 5);
    assert_eq!(r.right(), 11);
    assert_eq!(r.bottom(), 7);
}

#[test]
fn test_canvas_set_get() {
    let mut c = Canvas::new(5, 5, CharSet::Unicode);
    c.set(2, 3, 'X');
    assert_eq!(c.get(2, 3), 'X');
    assert_eq!(c.get(0, 0), ' ');
}

#[test]
fn test_canvas_set_out_of_bounds() {
    let mut c = Canvas::new(3, 3, CharSet::Unicode);
    // Should not panic
    c.set(10, 10, 'X');
    assert_eq!(c.get(10, 10), ' ');
}

#[test]
fn test_canvas_hline() {
    let mut c = Canvas::new(10, 5, CharSet::Ascii);
    c.hline(2, 1, 5, '-');
    for col in 1..=5 {
        assert_eq!(c.get(col, 2), '-');
    }
    assert_eq!(c.get(0, 2), ' ');
}

#[test]
fn test_canvas_vline() {
    let mut c = Canvas::new(10, 10, CharSet::Ascii);
    c.vline(3, 1, 4, '|');
    for row in 1..=4 {
        assert_eq!(c.get(3, row), '|');
    }
    assert_eq!(c.get(3, 0), ' ');
}

#[test]
fn test_canvas_set_merge_junction() {
    let mut c = Canvas::new(10, 10, CharSet::Unicode);
    // Place a horizontal line char, then merge a vertical — should produce cross
    c.set(5, 5, '─');
    c.set_merge(5, 5, '│');
    assert_eq!(c.get(5, 5), '┼');
}

#[test]
fn test_canvas_write_str() {
    let mut c = Canvas::new(20, 5, CharSet::Unicode);
    c.write_str(2, 1, "Hello");
    assert_eq!(c.get(2, 1), 'H');
    assert_eq!(c.get(6, 1), 'o');
}

#[test]
fn test_canvas_to_string_trims() {
    let mut c = Canvas::new(10, 3, CharSet::Ascii);
    c.set(0, 0, 'A');
    let s = c.to_string();
    let lines: Vec<&str> = s.lines().collect();
    assert_eq!(lines[0], "A");
    // trailing empty lines trimmed
    assert_eq!(lines.len(), 1);
    // ends with newline
    assert!(s.ends_with('\n'));
}

#[test]
fn test_canvas_draw_box() {
    let mut c = Canvas::new(10, 5, CharSet::Unicode);
    let bc = BoxChars::unicode();
    c.draw_box(Rect::new(0, 0, 5, 3), &bc);
    assert_eq!(c.get(0, 0), '┌');
    assert_eq!(c.get(4, 0), '┐');
    assert_eq!(c.get(0, 2), '└');
    assert_eq!(c.get(4, 2), '┘');
    assert_eq!(c.get(1, 0), '─');
    assert_eq!(c.get(0, 1), '│');
}
