//! A* pathfinder for edge routing on the character grid.
//!
//! Mirrors Python's layout/pathfinder.py.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};

use super::types::Point;

// ─── OccupancyGrid ───────────────────────────────────────────────────────────

/// 2D boolean grid tracking which cells are blocked by nodes.
pub struct OccupancyGrid {
    pub width: usize,
    pub height: usize,
    blocked: Vec<Vec<bool>>,
}

impl OccupancyGrid {
    pub fn create(width: usize, height: usize) -> Self {
        let blocked = vec![vec![false; width]; height];
        Self {
            width,
            height,
            blocked,
        }
    }

    /// Mark all cells inside a rectangle as blocked.
    pub fn mark_rect_blocked(&mut self, x: i64, y: i64, w: i64, h: i64) {
        let row_start = y.max(0) as usize;
        let row_end = ((y + h) as usize).min(self.height);
        let col_start = x.max(0) as usize;
        let col_end = ((x + w) as usize).min(self.width);
        for row in row_start..row_end {
            for col in col_start..col_end {
                self.blocked[row][col] = true;
            }
        }
    }

    pub fn is_free(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        let ux = x as usize;
        let uy = y as usize;
        if ux >= self.width || uy >= self.height {
            return false;
        }
        !self.blocked[uy][ux]
    }
}

// ─── Heuristic ───────────────────────────────────────────────────────────────

/// Manhattan distance + corner penalty (from reference repos).
fn heuristic(ax: i64, ay: i64, bx: i64, by: i64) -> i64 {
    let dx = (ax - bx).abs();
    let dy = (ay - by).abs();
    if dx == 0 || dy == 0 {
        dx + dy
    } else {
        dx + dy + 1
    }
}

// ─── A* Search ───────────────────────────────────────────────────────────────

/// 4-directional neighbors: (dx, dy)
const DIRS: [(i64, i64); 4] = [(0, 1), (0, -1), (1, 0), (-1, 0)];

/// Find shortest path from start to end on the grid, avoiding blocked cells.
///
/// The goal cell is allowed to be blocked (it's on a node border).
/// Returns list of Points from start to end, or None if no path found.
pub fn a_star(grid: &OccupancyGrid, start: Point, end: Point) -> Option<Vec<Point>> {
    let sx = start.x;
    let sy = start.y;
    let ex = end.x;
    let ey = end.y;

    // Min-heap: (Reverse(priority), Reverse(counter), x, y)
    let mut counter: u64 = 0;
    let mut open_set: BinaryHeap<(Reverse<i64>, Reverse<u64>, i64, i64)> = BinaryHeap::new();
    open_set.push((Reverse(heuristic(sx, sy, ex, ey)), Reverse(counter), sx, sy));

    let mut cost_so_far: HashMap<(i64, i64), i64> = HashMap::new();
    cost_so_far.insert((sx, sy), 0);

    let mut came_from: HashMap<(i64, i64), Option<(i64, i64)>> = HashMap::new();
    came_from.insert((sx, sy), None);

    while let Some((_, _, cx, cy)) = open_set.pop() {
        if cx == ex && cy == ey {
            // Reconstruct path
            let mut path = Vec::new();
            let mut cur: Option<(i64, i64)> = Some((cx, cy));
            while let Some((px, py)) = cur {
                path.push(Point::new(px, py));
                cur = came_from.get(&(px, py)).copied().flatten();
            }
            path.reverse();
            return Some(path);
        }

        let current_cost = *cost_so_far.get(&(cx, cy)).unwrap_or(&i64::MAX);

        for (dx, dy) in DIRS {
            let nx = cx + dx;
            let ny = cy + dy;

            // Allow stepping onto the goal even if blocked
            if nx == ex && ny == ey {
                // OK
            } else if !grid.is_free(nx, ny) {
                continue;
            }

            let new_cost = current_cost + 1;
            let key = (nx, ny);
            if !cost_so_far.contains_key(&key) || new_cost < cost_so_far[&key] {
                cost_so_far.insert(key, new_cost);
                let priority = new_cost + heuristic(nx, ny, ex, ey);
                counter += 1;
                open_set.push((Reverse(priority), Reverse(counter), nx, ny));
                came_from.insert(key, Some((cx, cy)));
            }
        }
    }

    None
}

// ─── Path Simplification ─────────────────────────────────────────────────────

/// Remove collinear intermediate points, keeping only direction changes.
pub fn simplify_path(path: Vec<Point>) -> Vec<Point> {
    if path.len() <= 2 {
        return path;
    }

    let mut result = vec![path[0].clone()];
    for i in 1..path.len() - 1 {
        let prev = &path[i - 1];
        let curr = &path[i];
        let nxt = &path[i + 1];
        let dx1 = curr.x - prev.x;
        let dy1 = curr.y - prev.y;
        let dx2 = nxt.x - curr.x;
        let dy2 = nxt.y - curr.y;
        if dx1 != dx2 || dy1 != dy2 {
            result.push(curr.clone());
        }
    }
    result.push(path.last().unwrap().clone());
    result
}
