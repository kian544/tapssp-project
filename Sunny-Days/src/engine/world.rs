use crate::engine::action::Action;
use crate::engine::entity::Player;
use crate::map::{generator::generate_rooms_and_corridors, tile::Tile, Map};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use std::collections::VecDeque;

#[derive(Clone)]
pub struct Level {
    pub map: Map,
    pub door: (i32, i32),
}

pub struct World {
    pub levels: Vec<Level>,     // exactly 2 rooms
    pub current: usize,         // 0 = Room 1, 1 = Room 2
    pub player: Player,
    pub logs: VecDeque<String>,
    pub seed: u64,
}

impl World {
    pub fn new(seed: u64, width: usize, height: usize) -> Self {
        // Room 1
        let (level0, spawn0) = Self::make_level(seed, 0, width, height);

        // Room 2
        let (level1, _spawn1) = Self::make_level(seed, 1, width, height);

        let mut logs = VecDeque::new();
        logs.push_back(format!("Seed: {}", seed));
        logs.push_back("Welcome to Sunny Days.".to_string());
        logs.push_back("Move with WASD or arrow keys. Press Q to quit.".to_string());
        logs.push_back("Find the white door to enter Room 2.".to_string());

        Self {
            levels: vec![level0, level1],
            current: 0,
            player: Player::new(spawn0.0, spawn0.1),
            logs,
            seed,
        }
    }

    fn current_level(&self) -> &Level {
        &self.levels[self.current]
    }

    pub fn current_map(&self) -> &Map {
        &self.current_level().map
    }

    fn make_level(
        base_seed: u64,
        depth: usize,
        width: usize,
        height: usize,
    ) -> (Level, (i32, i32)) {
        let seed = base_seed.wrapping_add(depth as u64 * 9_973);
        let mut map = generate_rooms_and_corridors(width, height, seed);

        // Spawn = first floor tile
        let (sx, sy) = map.find_first_floor().unwrap_or((1, 1));
        let spawn = (sx as i32, sy as i32);

        // Place exactly ONE door somewhere random, not on spawn
        let door = Self::place_random_door(&mut map, seed ^ 0xD00D, spawn);

        (
            Level { map, door },
            spawn,
        )
    }

    fn place_random_door(map: &mut Map, seed: u64, exclude: (i32, i32)) -> (i32, i32) {
        let mut floors: Vec<(i32, i32)> = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) == Tile::Floor {
                    floors.push((x as i32, y as i32));
                }
            }
        }

        let mut rng = StdRng::seed_from_u64(seed);

        let mut door = exclude;
        if floors.len() > 1 {
            loop {
                let idx = rng.gen_range(0..floors.len());
                let candidate = floors[idx];
                if candidate != exclude {
                    door = candidate;
                    break;
                }
            }
        }

        map.set(door.0 as usize, door.1 as usize, Tile::Door);
        door
    }

    pub fn push_log(&mut self, msg: impl Into<String>) {
        self.logs.push_back(msg.into());
        while self.logs.len() > 6 {
            self.logs.pop_front();
        }
    }

    fn toggle_room(&mut self) {
        let old_room = self.current;
        let new_room = if old_room == 0 { 1 } else { 0 };
        self.current = new_room;

        // Teleport player to the door location in the other room
        let target = self.levels[new_room].door;
        self.player.x = target.0;
        self.player.y = target.1;

        if new_room == 1 {
            self.push_log("You step through the door into Room 2...".to_string());
        } else {
            self.push_log("You step back into Room 1...".to_string());
        }
    }

    pub fn apply_action(&mut self, action: Action) -> bool {
        match action {
            Action::Move(dx, dy) => {
                let old = (self.player.x, self.player.y);

                let map_snapshot = self.current_map().clone();
                self.player.try_move(dx, dy, &map_snapshot);

                let newp = (self.player.x, self.player.y);
                if old != newp {
                    self.push_log(format!("Player moved to ({}, {})", newp.0, newp.1));
                }

                let tile = self.current_map().get(newp.0 as usize, newp.1 as usize);
                if tile == Tile::Door {
                    self.toggle_room();
                }

                true
            }
            Action::Quit => false,
            Action::None => true,
        }
    }
}
