pub mod generator;
pub mod tile;

use tile::Tile;

#[derive(Clone)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize, fill: Tile) -> Self {
        Self {
            width,
            height,
            tiles: vec![fill; width * height],
        }
    }

    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn get(&self, x: usize, y: usize) -> Tile {
        self.tiles[self.idx(x, y)]
    }

    pub fn set(&mut self, x: usize, y: usize, t: Tile) {
        let i = self.idx(x, y);
        self.tiles[i] = t;
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn find_first_floor(&self) -> Option<(usize, usize)> {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get(x, y) == Tile::Floor {
                    return Some((x, y));
                }
            }
        }
        None
    }

    pub fn is_walkable(&self, x: usize, y: usize) -> bool {
        // Door is no longer walkable; it acts like a character/NPC.
        matches!(self.get(x, y), Tile::Floor | Tile::Chest)
    }
}