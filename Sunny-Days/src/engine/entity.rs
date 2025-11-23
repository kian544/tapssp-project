use crate::map::Map;

#[derive(Debug, Clone)]
pub struct Equipment {
    pub name: String,
    pub atk: i32,
    pub def: i32,
}

#[derive(Debug, Clone)]
pub struct Consumable {
    pub name: String,
    pub heal: i32,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub sword: Option<Equipment>,
    pub shield: Option<Equipment>,
    pub consumables: Vec<Consumable>,
    pub selected: usize,
}

impl Inventory {
    /// Start with absolutely nothing equipped or carried.
    pub fn default_loadout() -> Self {
        Self {
            sword: None,
            shield: None,
            consumables: Vec::new(),
            selected: 0,
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = self.consumables.len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        let mut idx = self.selected as i32 + delta;
        if idx < 0 {
            idx = len as i32 - 1;
        } else if idx >= len as i32 {
            idx = 0;
        }
        self.selected = idx as usize;
    }

    pub fn take_selected(&mut self) -> Option<Consumable> {
        if self.consumables.is_empty() {
            return None;
        }
        if self.selected >= self.consumables.len() {
            self.selected = 0;
        }
        let item = self.consumables.remove(self.selected);
        if self.selected >= self.consumables.len() && !self.consumables.is_empty() {
            self.selected = self.consumables.len() - 1;
        }
        Some(item)
    }

    pub fn selected_consumable(&self) -> Option<&Consumable> {
        self.consumables.get(self.selected)
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub inventory: Inventory,
}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        let max_hp = 30;
        Self {
            x,
            y,
            hp: max_hp,
            max_hp,
            inventory: Inventory::default_loadout(),
        }
    }

    pub fn try_move(&mut self, dx: i32, dy: i32, map: &Map) {
        let nx = self.x + dx;
        let ny = self.y + dy;

        if nx < 0 || ny < 0 || nx >= map.width as i32 || ny >= map.height as i32 {
            return;
        }

        if map.is_walkable(nx as usize, ny as usize) {
            self.x = nx;
            self.y = ny;
        }
    }
}
