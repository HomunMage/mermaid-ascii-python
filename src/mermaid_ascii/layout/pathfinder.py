"""A* pathfinder for edge routing on the character grid."""

from __future__ import annotations

import heapq
from dataclasses import dataclass

from mermaid_ascii.layout.types import Point


@dataclass
class OccupancyGrid:
    """2D boolean grid tracking which cells are blocked by nodes."""

    width: int
    height: int
    blocked: list[list[bool]]

    @classmethod
    def create(cls, width: int, height: int) -> OccupancyGrid:
        blocked = [[False] * width for _ in range(height)]
        return cls(width=width, height=height, blocked=blocked)

    def mark_rect_blocked(self, x: int, y: int, w: int, h: int) -> None:
        """Mark all cells inside a rectangle as blocked."""
        for row in range(max(0, y), min(self.height, y + h)):
            for col in range(max(0, x), min(self.width, x + w)):
                self.blocked[row][col] = True

    def is_free(self, x: int, y: int) -> bool:
        if x < 0 or x >= self.width or y < 0 or y >= self.height:
            return False
        return not self.blocked[y][x]


def _heuristic(ax: int, ay: int, bx: int, by: int) -> int:
    """Manhattan distance + corner penalty (from reference repos)."""
    dx = abs(ax - bx)
    dy = abs(ay - by)
    if dx == 0 or dy == 0:
        return dx + dy
    return dx + dy + 1


# 4-directional neighbors
_DIRS = [(0, 1), (0, -1), (1, 0), (-1, 0)]


def a_star(grid: OccupancyGrid, start: Point, end: Point) -> list[Point] | None:
    """Find shortest path from start to end on the grid, avoiding blocked cells.

    The goal cell is allowed to be blocked (it's on a node border).
    Returns list of Points from start to end, or None if no path found.
    """
    sx, sy = start.x, start.y
    ex, ey = end.x, end.y

    # Priority queue: (priority, counter, x, y)
    counter = 0
    open_set: list[tuple[int, int, int, int]] = []
    heapq.heappush(open_set, (_heuristic(sx, sy, ex, ey), counter, sx, sy))

    cost_so_far: dict[tuple[int, int], int] = {(sx, sy): 0}
    came_from: dict[tuple[int, int], tuple[int, int] | None] = {(sx, sy): None}

    while open_set:
        _, _, cx, cy = heapq.heappop(open_set)

        if cx == ex and cy == ey:
            # Reconstruct path
            path: list[Point] = []
            cur: tuple[int, int] | None = (cx, cy)
            while cur is not None:
                path.append(Point(x=cur[0], y=cur[1]))
                cur = came_from[cur]
            path.reverse()
            return path

        current_cost = cost_so_far[(cx, cy)]

        for dx, dy in _DIRS:
            nx_, ny = cx + dx, cy + dy

            # Allow stepping onto the goal even if blocked
            if nx_ == ex and ny == ey:
                pass
            elif not grid.is_free(nx_, ny):
                continue

            new_cost = current_cost + 1
            key = (nx_, ny)
            if key not in cost_so_far or new_cost < cost_so_far[key]:
                cost_so_far[key] = new_cost
                priority = new_cost + _heuristic(nx_, ny, ex, ey)
                counter += 1
                heapq.heappush(open_set, (priority, counter, nx_, ny))
                came_from[key] = (cx, cy)

    return None


def simplify_path(path: list[Point]) -> list[Point]:
    """Remove collinear intermediate points, keeping only direction changes."""
    if len(path) <= 2:
        return list(path)

    result = [path[0]]
    for i in range(1, len(path) - 1):
        prev = path[i - 1]
        curr = path[i]
        nxt = path[i + 1]
        # Keep point if direction changes
        dx1 = curr.x - prev.x
        dy1 = curr.y - prev.y
        dx2 = nxt.x - curr.x
        dy2 = nxt.y - curr.y
        if dx1 != dx2 or dy1 != dy2:
            result.append(curr)
    result.append(path[-1])
    return result
