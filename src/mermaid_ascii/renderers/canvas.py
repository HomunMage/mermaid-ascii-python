"""Canvas — 2D character grid for rendering."""

from __future__ import annotations

from dataclasses import dataclass

from mermaid_ascii.renderers.charset import Arms, BoxChars, CharSet


@dataclass
class Rect:
    x: int
    y: int
    width: int
    height: int

    def right(self) -> int:
        return self.x + self.width

    def bottom(self) -> int:
        return self.y + self.height


class Canvas:
    """A 2D character grid onto which graph elements are painted."""

    def __init__(self, width: int, height: int, charset: CharSet) -> None:
        self.width = width
        self.height = height
        self.charset = charset
        self.cells: list[list[str]] = [[" "] * width for _ in range(height)]

    def get(self, col: int, row: int) -> str:
        if 0 <= row < self.height and 0 <= col < self.width:
            return self.cells[row][col]
        return " "

    def set(self, col: int, row: int, c: str) -> None:
        if 0 <= row < self.height and 0 <= col < self.width:
            self.cells[row][col] = c

    def set_merge(self, col: int, row: int, c: str) -> None:
        if row >= self.height or col >= self.width:
            return
        existing = self.cells[row][col]
        ea = Arms.from_char(existing)
        na = Arms.from_char(c)
        if ea is not None and na is not None:
            merged = ea.merge(na)
            self.cells[row][col] = merged.to_char(self.charset)
        else:
            self.cells[row][col] = c

    def hline(self, y: int, x1: int, x2: int, c: str) -> None:
        lo, hi = (x1, x2) if x1 <= x2 else (x2, x1)
        for col in range(lo, hi + 1):
            self.set_merge(col, y, c)

    def vline(self, x: int, y1: int, y2: int, c: str) -> None:
        lo, hi = (y1, y2) if y1 <= y2 else (y2, y1)
        for row in range(lo, hi + 1):
            self.set_merge(x, row, c)

    def draw_box(self, rect: Rect, bc: BoxChars) -> None:
        if rect.width < 2 or rect.height < 2:
            return
        x0 = rect.x
        y0 = rect.y
        x1 = rect.x + rect.width - 1
        y1 = rect.y + rect.height - 1
        self.set(x0, y0, bc.top_left)
        self.set(x1, y0, bc.top_right)
        self.set(x0, y1, bc.bottom_left)
        self.set(x1, y1, bc.bottom_right)
        for col in range(x0 + 1, x1):
            self.set(col, y0, bc.horizontal)
            self.set(col, y1, bc.horizontal)
        for row in range(y0 + 1, y1):
            self.set(x0, row, bc.vertical)
            self.set(x1, row, bc.vertical)

    def write_str(self, col: int, row: int, s: str) -> None:
        for i, ch in enumerate(s):
            c = col + i
            if c >= self.width or row >= self.height:
                break
            self.cells[row][c] = ch

    def to_string(self) -> str:
        lines = []
        for row in self.cells:
            line = "".join(row).rstrip()
            lines.append(line)
        out = "\n".join(lines)
        trimmed = out.rstrip("\n")
        return trimmed + "\n"
