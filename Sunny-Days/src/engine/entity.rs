use crate::map::Map;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Sword,
    Shield,
}

#[derive(Debug, Clone)]
pub struct Equipment {
    pub name: String,
    pub slot: EquipSlot,     // NEW: tells backpack where it equips
    pub atk_bonus: i32,
    pub def_bonus: i32,
    pub speed_bonus: i32,
}

#[derive(Debug, Clone)]
pub struct Consumable {
    pub name: String,
    pub heal: i32,
    pub atk_bonus: i32,
    pub def_bonus: i32,
}

#[derive(Debug, Clone)]
pub struct TempBuff {
    pub atk_bonus: i32,
    pub def_bonus: i32,
    pub speed_bonus: i32,
    pub expires_at: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvTab {
    Weapons,
    Consumables,
    Backpack,
}

#[derive(Debug, Clone)]
pub struct Inventory {
    pub sword: Option<Equipment>,
    pub shield: Option<Equipment>,

    pub consumables: Vec<Consumable>, // up to 10 later
    pub backpack: Vec<Equipment>,     // unequipped gear

    pub tab: InvTab,
    pub weapon_cursor: usize,      // 0 sword, 1 shield
    pub consumable_cursor: usize,  // 0..len-1
    pub backpack_cursor: usize,    // 0..len-1
}

#[derive(Debug, Clone)]
pub enum InvSelection {
    SwordSlot,
    ShieldSlot,
    Consumable(usize),
    BackpackItem(usize),
    None,
}

impl Inventory {
    pub fn default_loadout() -> Self {
        Self {
            sword: None,
            shield: None,
            consumables: Vec::new(),
            backpack: Vec::new(),
            tab: InvTab::Weapons,
            weapon_cursor: 0,
            consumable_cursor: 0,
            backpack_cursor: 0,
        }
    }

    pub fn toggle_tab(&mut self) {
        self.tab = match self.tab {
            InvTab::Weapons => InvTab::Consumables,
            InvTab::Consumables => InvTab::Backpack,
            InvTab::Backpack => InvTab::Weapons,
        };

        if self.weapon_cursor > 1 {
            self.weapon_cursor = 1;
        }
        if !self.consumables.is_empty() && self.consumable_cursor >= self.consumables.len() {
            self.consumable_cursor = self.consumables.len() - 1;
        }
        if !self.backpack.is_empty() && self.backpack_cursor >= self.backpack.len() {
            self.backpack_cursor = self.backpack.len() - 1;
        }
    }

    pub fn move_cursor(&mut self, delta: i32) {
        match self.tab {
            InvTab::Weapons => {
                let len = 2;
                let mut idx = self.weapon_cursor as i32 + delta;
                if idx < 0 {
                    idx = len as i32 - 1;
                } else if idx >= len as i32 {
                    idx = 0;
                }
                self.weapon_cursor = idx as usize;
            }

            InvTab::Consumables => {
                let len = self.consumables.len();
                if len == 0 {
                    self.consumable_cursor = 0;
                    return;
                }
                let mut idx = self.consumable_cursor as i32 + delta;
                if idx < 0 {
                    idx = len as i32 - 1;
                } else if idx >= len as i32 {
                    idx = 0;
                }
                self.consumable_cursor = idx as usize;
            }

            InvTab::Backpack => {
                let len = self.backpack.len();
                if len == 0 {
                    self.backpack_cursor = 0;
                    return;
                }
                let mut idx = self.backpack_cursor as i32 + delta;
                if idx < 0 {
                    idx = len as i32 - 1;
                } else if idx >= len as i32 {
                    idx = 0;
                }
                self.backpack_cursor = idx as usize;
            }
        }
    }

    pub fn selection(&self) -> InvSelection {
        match self.tab {
            InvTab::Weapons => {
                if self.weapon_cursor == 0 {
                    InvSelection::SwordSlot
                } else {
                    InvSelection::ShieldSlot
                }
            }
            InvTab::Consumables => {
                if self.consumables.is_empty() {
                    InvSelection::None
                } else {
                    InvSelection::Consumable(self.consumable_cursor)
                }
            }
            InvTab::Backpack => {
                if self.backpack.is_empty() {
                    InvSelection::None
                } else {
                    InvSelection::BackpackItem(self.backpack_cursor)
                }
            }
        }
    }

    pub fn take_selected_consumable(&mut self) -> Option<Consumable> {
        if self.tab != InvTab::Consumables {
            return None;
        }
        if self.consumables.is_empty() {
            return None;
        }
        let idx = self.consumable_cursor.min(self.consumables.len() - 1);
        Some(self.consumables.remove(idx))
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub x: i32,
    pub y: i32,

    pub hp: i32,
    pub max_hp: i32,

    pub base_attack: i32,
    pub base_defense: i32,
    pub base_speed: i32,

    pub inventory: Inventory,
    pub buffs: Vec<TempBuff>,

}

impl Player {
    pub fn new(x: i32, y: i32) -> Self {
        let max_hp = 30;
        Self {
            x,
            y,
            hp: max_hp,
            max_hp,
            base_attack: 10,
            base_defense: 8,
            base_speed: 5,
            inventory: Inventory::default_loadout(),
            buffs: Vec::new(),
        }
    }

    pub fn add_temp_buff(&mut self, atk: i32, def: i32, speed: i32, duration: Duration) {
    if atk == 0 && def == 0 && speed == 0 {
        return;
    }
    self.buffs.push(TempBuff {
        atk_bonus: atk,
        def_bonus: def,
        speed_bonus: speed,
        expires_at: Instant::now() + duration,
    });
}

    pub fn purge_expired_buffs(&mut self) {
        let now = Instant::now();
        self.buffs.retain(|b| b.expires_at > now);
    }

    fn active_buff_sums(&self) -> (i32, i32, i32) {
        let now = Instant::now();
        let mut atk = 0;
        let mut def = 0;
        let mut spd = 0;

        for b in &self.buffs {
            if b.expires_at > now {
                atk += b.atk_bonus;
                def += b.def_bonus;
                spd += b.speed_bonus;
            }
        }
        (atk, def, spd)
    }


    pub fn attack(&self) -> i32 {
        let mut v = self.base_attack;
        if let Some(sw) = &self.inventory.sword {
            v += sw.atk_bonus;
        }
        if let Some(sh) = &self.inventory.shield {
            v += sh.atk_bonus;
        }
        let (atk_b, _, _) = self.active_buff_sums();
        v += atk_b;
        v
    }


    pub fn defense(&self) -> i32 {
        let mut v = self.base_defense;
        if let Some(sw) = &self.inventory.sword {
            v += sw.def_bonus;
        }
        if let Some(sh) = &self.inventory.shield {
            v += sh.def_bonus;
        }
        let (_, def_b, _) = self.active_buff_sums();
        v += def_b;
        v
    }


    pub fn speed(&self) -> i32 {
        let mut v = self.base_speed;
        if let Some(sw) = &self.inventory.sword {
            v += sw.speed_bonus;
        }
        if let Some(sh) = &self.inventory.shield {
            v += sh.speed_bonus;
        }
        let (_, _, spd_b) = self.active_buff_sums();
        v += spd_b;
        v
    }


    pub fn equip_sword(&mut self, eq: Equipment) {
        self.inventory.sword = Some(eq);
    }

    pub fn equip_shield(&mut self, eq: Equipment) {
        self.inventory.shield = Some(eq);
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
