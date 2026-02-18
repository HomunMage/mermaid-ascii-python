use super::*;

#[test]
fn test_arms_from_char_horizontal() {
    let a = Arms::from_char('─').unwrap();
    assert!(!a.up);
    assert!(!a.down);
    assert!(a.left);
    assert!(a.right);
}

#[test]
fn test_arms_from_char_vertical() {
    let a = Arms::from_char('│').unwrap();
    assert!(a.up);
    assert!(a.down);
    assert!(!a.left);
    assert!(!a.right);
}

#[test]
fn test_arms_from_char_unknown() {
    assert!(Arms::from_char('X').is_none());
    assert!(Arms::from_char(' ').is_none());
}

#[test]
fn test_arms_merge() {
    let a = Arms::new(true, false, false, true);
    let b = Arms::new(false, true, true, false);
    let merged = a.merge(b);
    assert_eq!(merged, Arms::new(true, true, true, true));
}

#[test]
fn test_arms_to_char_unicode() {
    assert_eq!(
        Arms::new(true, true, true, true).to_char(CharSet::Unicode),
        '┼'
    );
    assert_eq!(
        Arms::new(false, false, true, true).to_char(CharSet::Unicode),
        '─'
    );
    assert_eq!(
        Arms::new(true, true, false, false).to_char(CharSet::Unicode),
        '│'
    );
}

#[test]
fn test_arms_to_char_ascii() {
    assert_eq!(
        Arms::new(true, true, true, true).to_char(CharSet::Ascii),
        '+'
    );
    assert_eq!(
        Arms::new(false, false, true, true).to_char(CharSet::Ascii),
        '-'
    );
    assert_eq!(
        Arms::new(true, true, false, false).to_char(CharSet::Ascii),
        '|'
    );
}

#[test]
fn test_boxchars_unicode() {
    let bc = BoxChars::unicode();
    assert_eq!(bc.horizontal, '─');
    assert_eq!(bc.vertical, '│');
    assert_eq!(bc.top_left, '┌');
    assert_eq!(bc.cross, '┼');
}

#[test]
fn test_boxchars_ascii() {
    let bc = BoxChars::ascii();
    assert_eq!(bc.horizontal, '-');
    assert_eq!(bc.vertical, '|');
    assert_eq!(bc.top_left, '+');
    assert_eq!(bc.cross, '+');
}
