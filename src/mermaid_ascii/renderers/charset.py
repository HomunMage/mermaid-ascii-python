"""Character sets and junction merging for box-drawing."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum


class CharSet(Enum):
    Unicode = "unicode"
    Ascii = "ascii"


@dataclass
class BoxChars:
    top_left: str
    top_right: str
    bottom_left: str
    bottom_right: str
    horizontal: str
    vertical: str
    tee_right: str
    tee_left: str
    tee_down: str
    tee_up: str
    cross: str
    arrow_right: str
    arrow_left: str
    arrow_down: str
    arrow_up: str

    @classmethod
    def unicode(cls) -> BoxChars:
        return cls(
            top_left="┌",
            top_right="┐",
            bottom_left="└",
            bottom_right="┘",
            horizontal="─",
            vertical="│",
            tee_right="├",
            tee_left="┤",
            tee_down="┬",
            tee_up="┴",
            cross="┼",
            arrow_right="►",
            arrow_left="◄",
            arrow_down="▼",
            arrow_up="▲",
        )

    @classmethod
    def ascii(cls) -> BoxChars:
        return cls(
            top_left="+",
            top_right="+",
            bottom_left="+",
            bottom_right="+",
            horizontal="-",
            vertical="|",
            tee_right="+",
            tee_left="+",
            tee_down="+",
            tee_up="+",
            cross="+",
            arrow_right=">",
            arrow_left="<",
            arrow_down="v",
            arrow_up="^",
        )

    @classmethod
    def for_charset(cls, cs: CharSet) -> BoxChars:
        if cs == CharSet.Unicode:
            return cls.unicode()
        return cls.ascii()


@dataclass
class Arms:
    """Which arms of a junction cell are active."""

    up: bool = False
    down: bool = False
    left: bool = False
    right: bool = False

    @classmethod
    def from_char(cls, c: str) -> Arms | None:
        table: dict[str, tuple[bool, bool, bool, bool]] = {
            "─": (False, False, True, True),
            "│": (True, True, False, False),
            "┌": (False, True, False, True),
            "┐": (False, True, True, False),
            "└": (True, False, False, True),
            "┘": (True, False, True, False),
            "├": (True, True, False, True),
            "┤": (True, True, True, False),
            "┬": (False, True, True, True),
            "┴": (True, False, True, True),
            "┼": (True, True, True, True),
            "-": (False, False, True, True),
            "|": (True, True, False, False),
            "+": (True, True, True, True),
        }
        entry = table.get(c)
        if entry is None:
            return None
        u, d, lft, r = entry
        return cls(up=u, down=d, left=lft, right=r)

    def merge(self, other: Arms) -> Arms:
        return Arms(
            up=self.up or other.up,
            down=self.down or other.down,
            left=self.left or other.left,
            right=self.right or other.right,
        )

    def to_char(self, cs: CharSet) -> str:
        bc = BoxChars.for_charset(cs)
        key = (self.up, self.down, self.left, self.right)
        match key:
            case (False, False, False, False):
                return " "
            case (False, False, True, True):
                return bc.horizontal
            case (True, True, False, False):
                return bc.vertical
            case (False, True, False, True):
                return bc.top_left
            case (False, True, True, False):
                return bc.top_right
            case (True, False, False, True):
                return bc.bottom_left
            case (True, False, True, False):
                return bc.bottom_right
            case (True, True, False, True):
                return bc.tee_right
            case (True, True, True, False):
                return bc.tee_left
            case (False, True, True, True):
                return bc.tee_down
            case (True, False, True, True):
                return bc.tee_up
            case (True, True, True, True):
                return bc.cross
            case (True, False, False, False):
                return bc.vertical
            case (False, True, False, False):
                return bc.vertical
            case (False, False, True, False):
                return bc.horizontal
            case (False, False, False, True):
                return bc.horizontal
            case _:
                return " "
