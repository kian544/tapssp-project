use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use crate::map::{Map, tile::Tile};

#[derive(Clone, Copy)]
struct Rect {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}
impl Rect {
    fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
    fn intersects(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 &&
        self.y1 <= other.y2 && self.y2 >= other.y1
    }
}

/// Generate rooms + corridors. Corridors are guaranteed width >= 2 tiles.
pub fn generate_rooms_and_corridors(width: usize, height: usize, seed: u64) -> Map {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = Map::new(width, height, Tile::Wall);

    let max_rooms = 10;
    let mut rooms: Vec<Rect> = Vec::new();

    for _ in 0..max_rooms {
        let w = rng.gen_range(6..=12);
        let h = rng.gen_range(6..=10);

        if width <= w + 4 || height <= h + 4 { break; }

        let x = rng.gen_range(2..(width - w - 2));
        let y = rng.gen_range(2..(height - h - 2));

        // Inflate room slightly so we enforce clearance between rooms/hallways
        let new_room = Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        };

        // Reject if too close to another room
        let mut ok = true;
        for r in &rooms {
            // expanded "buffer" of 2 tiles around existing rooms
            let buffered = Rect {
                x1: r.x1.saturating_sub(2),
                y1: r.y1.saturating_sub(2),
                x2: (r.x2 + 2).min(width - 1),
                y2: (r.y2 + 2).min(height - 1),
            };
            if new_room.intersects(&buffered) {
                ok = false;
                break;
            }
        }
        if !ok { continue; }

        carve_room(&mut map, new_room);

        if let Some(prev) = rooms.last() {
            let (px, py) = prev.center();
            let (nx, ny) = new_room.center();

            if rng.gen_bool(0.5) {
                carve_h_corridor2(&mut map, px, nx, py);
                carve_v_corridor2(&mut map, py, ny, nx);
            } else {
                carve_v_corridor2(&mut map, py, ny, px);
                carve_h_corridor2(&mut map, px, nx, ny);
            }
        }

        rooms.push(new_room);
    }

    map
}

fn carve_room(map: &mut Map, r: Rect) {
    for y in r.y1..=r.y2 {
        for x in r.x1..=r.x2 {
            map.set(x, y, Tile::Floor);
        }
    }
}

/// 2-tile wide horizontal corridor (y fixed)
fn carve_h_corridor2(map: &mut Map, x1: usize, x2: usize, y: usize) {
    let (start, end) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
    for x in start..=end {
        map.set(x, y, Tile::Floor);
        if y + 1 < map.height {
            map.set(x, y + 1, Tile::Floor);
        }
    }
}

/// 2-tile wide vertical corridor (x fixed)
fn carve_v_corridor2(map: &mut Map, y1: usize, y2: usize, x: usize) {
    let (start, end) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
    for y in start..=end {
        map.set(x, y, Tile::Floor);
        if x + 1 < map.width {
            map.set(x + 1, y, Tile::Floor);
        }
    }
}
