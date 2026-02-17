"""Tests for render.py — port of 12 Rust canvas/arms tests."""

from mermaid_ascii.render import (
    Arms,
    BoxChars,
    Canvas,
    CharSet,
    Rect,
    flip_horizontal,
    flip_vertical,
    remap_char_horizontal,
    remap_char_vertical,
)


class TestArmsFromChar:
    def test_from_char_horizontal(self):
        a = Arms.from_char("─")
        assert a is not None
        assert not a.up and not a.down and a.left and a.right

    def test_from_char_vertical(self):
        a = Arms.from_char("│")
        assert a is not None
        assert a.up and a.down and not a.left and not a.right

    def test_from_char_unknown_returns_none(self):
        assert Arms.from_char("X") is None
        assert Arms.from_char("A") is None
        assert Arms.from_char(" ") is None

    def test_from_char_ascii_pipe(self):
        a = Arms.from_char("|")
        assert a is not None
        assert a.up and a.down and not a.left and not a.right

    def test_from_char_plus(self):
        a = Arms.from_char("+")
        assert a is not None
        assert a.up and a.down and a.left and a.right


class TestArmsMerge:
    def test_merge_cross(self):
        horiz = Arms.from_char("─")
        vert = Arms.from_char("│")
        assert horiz is not None and vert is not None
        merged = horiz.merge(vert)
        assert merged.to_char(CharSet.Unicode) == "┼"

    def test_merge_tee_right(self):
        # Vertical + right arm → tee pointing right (├)
        vert = Arms(up=True, down=True, left=False, right=False)
        right = Arms(up=False, down=False, left=False, right=True)
        merged = vert.merge(right)
        assert merged.to_char(CharSet.Unicode) == "├"

    def test_to_char_ascii_cross(self):
        cross = Arms(up=True, down=True, left=True, right=True)
        assert cross.to_char(CharSet.Ascii) == "+"

    def test_merge_corner_top_left(self):
        # down + right → ┌
        a = Arms(up=False, down=True, left=False, right=True)
        assert a.to_char(CharSet.Unicode) == "┌"

    def test_merge_idempotent(self):
        a = Arms(up=True, down=False, left=True, right=False)
        merged = a.merge(a)
        assert merged.up == a.up
        assert merged.left == a.left


class TestCanvasBasics:
    def test_set_get(self):
        canvas = Canvas(10, 5, CharSet.Unicode)
        canvas.set(3, 2, "X")
        assert canvas.get(3, 2) == "X"
        assert canvas.get(0, 0) == " "

    def test_set_out_of_bounds_no_panic(self):
        canvas = Canvas(5, 5, CharSet.Unicode)
        canvas.set(10, 10, "X")  # should not raise
        assert canvas.get(10, 10) == " "

    def test_set_merge_junction(self):
        canvas = Canvas(10, 10, CharSet.Unicode)
        canvas.set(5, 5, "─")
        canvas.set_merge(5, 5, "│")
        assert canvas.get(5, 5) == "┼"

    def test_hline(self):
        canvas = Canvas(20, 5, CharSet.Unicode)
        canvas.hline(2, 3, 7, "─")
        for col in range(3, 8):
            assert canvas.get(col, 2) == "─", f"col={col}"
        assert canvas.get(2, 2) == " "
        assert canvas.get(8, 2) == " "

    def test_vline(self):
        canvas = Canvas(10, 20, CharSet.Unicode)
        canvas.vline(4, 2, 8, "│")
        for row in range(2, 9):
            assert canvas.get(4, row) == "│", f"row={row}"

    def test_draw_box(self):
        canvas = Canvas(20, 10, CharSet.Unicode)
        bc = BoxChars.unicode()
        rect = Rect(2, 1, 6, 3)
        canvas.draw_box(rect, bc)

        # Corners.
        assert canvas.get(2, 1) == "┌"
        assert canvas.get(7, 1) == "┐"
        assert canvas.get(2, 3) == "└"
        assert canvas.get(7, 3) == "┘"

        # Top edge.
        for col in range(3, 7):
            assert canvas.get(col, 1) == "─", f"top col={col}"

        # Left edge.
        assert canvas.get(2, 2) == "│"
        # Right edge.
        assert canvas.get(7, 2) == "│"

    def test_write_str(self):
        canvas = Canvas(20, 5, CharSet.Unicode)
        canvas.write_str(3, 2, "hello")
        assert canvas.get(3, 2) == "h"
        assert canvas.get(4, 2) == "e"
        assert canvas.get(7, 2) == "o"

    def test_to_string_trims_trailing_spaces(self):
        canvas = Canvas(10, 3, CharSet.Unicode)
        canvas.set(0, 0, "A")
        s = canvas.to_string()
        lines = s.splitlines()
        assert lines[0] == "A"  # trailing spaces stripped

    def test_hline_vline_junction_merge(self):
        # Drawing a horizontal line then a vertical line crossing it should
        # produce a ┼ at the intersection.
        canvas = Canvas(20, 20, CharSet.Unicode)
        canvas.hline(5, 2, 10, "─")
        canvas.vline(6, 2, 10, "│")
        # At (6, 5) we have both h and v — should be ┼.
        assert canvas.get(6, 5) == "┼"
        # At (6, 3) — only v before h crosses.
        assert canvas.get(6, 3) == "│"

    def test_rect_right_bottom(self):
        r = Rect(3, 4, 10, 5)
        assert r.right() == 13
        assert r.bottom() == 9


class TestDirectionTransforms:
    def test_remap_char_vertical_arrows(self):
        assert remap_char_vertical("▼") == "▲"
        assert remap_char_vertical("▲") == "▼"
        assert remap_char_vertical("v") == "^"
        assert remap_char_vertical("^") == "v"

    def test_remap_char_vertical_corners(self):
        assert remap_char_vertical("┌") == "└"
        assert remap_char_vertical("└") == "┌"
        assert remap_char_vertical("┐") == "┘"
        assert remap_char_vertical("┘") == "┐"

    def test_remap_char_horizontal_arrows(self):
        assert remap_char_horizontal("►") == "◄"
        assert remap_char_horizontal("◄") == "►"
        assert remap_char_horizontal(">") == "<"
        assert remap_char_horizontal("<") == ">"

    def test_flip_vertical_reverses_rows(self):
        s = "A\nB\nC\n"
        result = flip_vertical(s)
        lines = result.splitlines()
        assert lines[0] == "C"
        assert lines[1] == "B"
        assert lines[2] == "A"

    def test_flip_horizontal_reverses_cols(self):
        s = "ABC\n"
        result = flip_horizontal(s)
        lines = result.splitlines()
        assert lines[0] == "CBA"
