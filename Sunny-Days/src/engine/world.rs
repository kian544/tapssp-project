use crate::engine::action::Action;
use crate::engine::entity::{
    Equipment, Player, InvSelection, InvTab, Consumable, EquipSlot as Slot,
};
use crate::map::{generator::generate_rooms_and_corridors, tile::Tile, Map};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use std::collections::VecDeque;
use std::time::Duration;

#[derive(Clone)]
pub struct Chest {
    pub x: i32,
    pub y: i32,
    pub item: Option<Consumable>,
    pub weapon: Option<Equipment>,
    pub opened: bool,
}

#[derive(Clone)]
pub struct Level {
    pub map: Map,
    pub door: (i32, i32),
    pub chests: Vec<Chest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Title,
    Intro,
    Playing,
    Dialogue,
    Battle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcId {
    MayorSol,
    Noor,
    Lamp,
    Random1,
    Random2,
    Random3,
    Weeping1,
    Weeping2,
    Weeping3,
    Weeping4,
    Shab,
    Krad,
    Mah,
}

#[derive(Debug, Clone)]
pub struct Npc {
    pub id: NpcId,
    pub name: String,
    pub room: usize,
    pub x: i32,
    pub y: i32,
    pub symbol: char,
}

#[derive(Debug, Clone)]
pub enum AwaitingChoice {
    YesNoMayor,
    ABNoorWeapon,
    Chest {
        room: usize,
        x: i32,
        y: i32,
        item: Option<Consumable>,
        weapon: Option<Equipment>,
    },
}

#[derive(Debug, Clone)]
pub struct DialogueSession {
    pub npc: NpcId,
    pub title: String,
    pub pages: Vec<String>,
    pub page_index: usize,
    pub awaiting: Option<AwaitingChoice>,
}

#[derive(Debug, Clone)]
pub struct BattleSession {
    pub enemy_id: NpcId,
    pub enemy_name: String,
    pub enemy_hp: i32,
    pub enemy_max_hp: i32,
    pub enemy_atk: i32,
    pub enemy_def: i32,
    pub enemy_speed: i32,
    
    pub penalty_mode: bool,
    pub player_initiated: bool,
}

pub struct World {
    pub levels: Vec<Level>,
    pub current: usize,
    pub player: Player,

    pub logs: VecDeque<String>,
    pub seed: u64,

    pub inventory_open: bool,
    pub stats_open: bool,
    pub state: GameState,

    intro_lines: Vec<String>,

    pub npcs: Vec<Npc>,
    
    mayor_done: bool,
    noor_done: bool,
    lamp_done: bool,
    
    shab_defeated: bool,
    krad_defeated: bool,
    mah_defeated: bool,

    pub dialogue: Option<DialogueSession>,
    pub battle: Option<BattleSession>,
}

impl World {
    const NPC_MIN_SEP: i32 = 5;

    pub fn new(seed: u64, width: usize, height: usize) -> Self {
        let (level0, spawn0) = Self::make_level(seed, 0, width, height);
        let (level1, _spawn1) = Self::make_level(seed, 1, width, height);

        let mut logs = VecDeque::new();
        logs.push_back(format!("Seed: {}", seed));
        logs.push_back("Welcome to Sunny Day(s).".to_string());
        logs.push_back("Move with WASD or arrow keys.".to_string());
        logs.push_back("Press E to talk to NPCs / open chests.".to_string());
        logs.push_back("Press I to open inventory.".to_string());
        logs.push_back("Press T to toggle inventory tabs.".to_string());
        logs.push_back("Press Q to open stats.".to_string());

        let intro_lines = vec![
            "Welcome to the Sunny Day, where everything was once bright".to_string(),
            "and happy, is now in despair.".to_string(),
            "".to_string(),
            "It is up to you, to bring sunny times back.".to_string(),
            "Listen to its people, understand your mission.".to_string(),
        ];

        let mut world = Self {
            levels: vec![level0, level1],
            current: 0,
            player: Player::new(spawn0.0, spawn0.1),

            logs,
            seed,

            inventory_open: false,
            stats_open: false,
            state: GameState::Title,

            intro_lines,

            npcs: Vec::new(),
            mayor_done: false,
            noor_done: false,
            lamp_done: false,
            shab_defeated: false,
            krad_defeated: false,
            mah_defeated: false,

            dialogue: None,
            battle: None,
        };

        world.spawn_npcs(spawn0);
        world
    }

    fn fmt_hp_delta(delta: i32) -> String {
        if delta >= 0 { format!("+{} HP", delta) } else { format!("{} HP", delta) }
    }

    fn random_floor_spaced(&self, room: usize, taken: &[(i32, i32)], min_dist: i32) -> (i32, i32) {
        let map = &self.levels[room].map;
        let mut floors = Vec::new();

        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) == Tile::Floor {
                    let p = (x as i32, y as i32);
                    let ok = taken.iter().all(|&(tx, ty)| {
                        (tx - p.0).abs().max((ty - p.1).abs()) >= min_dist
                    });
                    if ok { floors.push(p); }
                }
            }
        }

        if floors.is_empty() {
            return self.random_floor_excluding(room, taken);
        }

        let mut rng = StdRng::seed_from_u64(self.seed ^ 0xBEEFu64 ^ (taken.len() as u64 * 31));
        floors[rng.gen_range(0..floors.len())]
    }

    fn spawn_npcs(&mut self, spawn0: (i32, i32)) {
        // --- ROOM 1 ---
        let mut mx = spawn0.0 + 5;
        let mut my = spawn0.1;
        if !self.is_floor(0, mx, my) {
            let candidates = [(mx, my), (mx, my + 1), (mx, my - 1), (mx + 1, my), (mx - 1, my)];
            for (cx, cy) in candidates {
                if self.is_floor(0, cx, cy) { mx = cx; my = cy; break; }
            }
        }
        self.npcs.push(Npc { id: NpcId::MayorSol, name: "Mayor Sol".to_string(), room: 0, x: mx, y: my, symbol: 'M' });

        let mut taken_r0: Vec<(i32, i32)> = vec![(spawn0.0, spawn0.1), (mx, my), self.levels[0].door];
        for ch in &self.levels[0].chests { taken_r0.push((ch.x, ch.y)); }

        for (id, sym, name) in [(NpcId::Noor, 'N', "Noor"), (NpcId::Lamp, 'L', "Lamp")] {
            let (x, y) = self.random_floor_spaced(0, &taken_r0, Self::NPC_MIN_SEP);
            taken_r0.push((x, y));
            self.npcs.push(Npc { id, name: name.to_string(), room: 0, x, y, symbol: sym });
        }

        for id in [NpcId::Random1, NpcId::Random2, NpcId::Random3] {
            let (vx, vy) = self.random_floor_spaced(0, &taken_r0, Self::NPC_MIN_SEP);
            taken_r0.push((vx, vy));
            self.npcs.push(Npc { id, name: "Villager".to_string(), room: 0, x: vx, y: vy, symbol: '●' });
        }

        // --- ROOM 2 ---
        let mut taken_r1: Vec<(i32, i32)> = vec![self.levels[1].door];
        for ch in &self.levels[1].chests { taken_r1.push((ch.x, ch.y)); }

        for id in [NpcId::Weeping1, NpcId::Weeping2, NpcId::Weeping3, NpcId::Weeping4] {
            let (wx, wy) = self.random_floor_spaced(1, &taken_r1, Self::NPC_MIN_SEP);
            taken_r1.push((wx, wy));
            self.npcs.push(Npc { id, name: "Weeping Villager".to_string(), room: 1, x: wx, y: wy, symbol: '●' });
        }

        let (sx, sy) = self.random_floor_spaced(1, &taken_r1, Self::NPC_MIN_SEP);
        taken_r1.push((sx, sy));
        self.npcs.push(Npc { id: NpcId::Shab, name: "Shab".to_string(), room: 1, x: sx, y: sy, symbol: 'S' });

        let (kx, ky) = self.random_floor_spaced(1, &taken_r1, Self::NPC_MIN_SEP);
        taken_r1.push((kx, ky));
        self.npcs.push(Npc { id: NpcId::Krad, name: "Krad".to_string(), room: 1, x: kx, y: ky, symbol: 'K' });

        let (bx, by) = self.random_floor_spaced(1, &taken_r1, Self::NPC_MIN_SEP);
        taken_r1.push((bx, by));
        self.npcs.push(Npc { id: NpcId::Mah, name: "Mah".to_string(), room: 1, x: bx, y: by, symbol: 'M' });
    }

    fn is_floor(&self, room: usize, x: i32, y: i32) -> bool {
        let map = &self.levels[room].map;
        if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 { return false; }
        map.get(x as usize, y as usize) == Tile::Floor
    }

    fn random_floor_excluding(&self, room: usize, exclude: &[(i32, i32)]) -> (i32, i32) {
        let map = &self.levels[room].map;
        let mut floors = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) == Tile::Floor {
                    let p = (x as i32, y as i32);
                    if !exclude.contains(&p) { floors.push(p); }
                }
            }
        }
        let mut rng = StdRng::seed_from_u64(self.seed ^ 0xBEEFu64);
        floors[rng.gen_range(0..floors.len())]
    }

    pub fn intro_lines(&self) -> &[String] { &self.intro_lines }
    fn current_level(&self) -> &Level { &self.levels[self.current] }
    fn current_level_mut(&mut self) -> &mut Level { &mut self.levels[self.current] }
    pub fn current_map(&self) -> &Map { &self.current_level().map }
    pub fn npc_at(&self, room: usize, x: i32, y: i32) -> Option<&Npc> {
        self.npcs.iter().find(|n| n.room == room && n.x == x && n.y == y)
    }
    pub fn npc_near_player(&self) -> Option<&Npc> {
        let px = self.player.x;
        let py = self.player.y;
        self.npcs.iter().find(|n| n.room == self.current && (n.x - px).abs().max((n.y - py).abs()) <= 1)
    }

    fn make_level(base_seed: u64, depth: usize, width: usize, height: usize) -> (Level, (i32, i32)) {
        let seed = base_seed.wrapping_add(depth as u64 * 9_973);
        let mut map = generate_rooms_and_corridors(width, height, seed);
        let (sx, sy) = map.find_first_floor().unwrap_or((1, 1));
        let spawn = (sx as i32, sy as i32);
        let door = Self::place_random_door(&mut map, seed ^ 0xD00D, spawn);
        
        // Random chests for consumables (in Room 1 and Room 2 now)
        let chests = Self::scatter_chests(&mut map, seed ^ 0xC1E57, spawn, door);
        
        (Level { map, door, chests }, spawn)
    }

    fn place_random_door(map: &mut Map, seed: u64, exclude: (i32, i32)) -> (i32, i32) {
        let mut floors = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) == Tile::Floor { floors.push((x as i32, y as i32)); }
            }
        }
        let mut rng = StdRng::seed_from_u64(seed);
        let mut door = exclude;
        if floors.len() > 1 {
            loop {
                let idx = rng.gen_range(0..floors.len());
                if floors[idx] != exclude { door = floors[idx]; break; }
            }
        }
        map.set(door.0 as usize, door.1 as usize, Tile::Door);
        door
    }

    fn random_consumable(rng: &mut StdRng) -> Consumable {
        match rng.gen_range(0..4) {
            0 => Consumable { name: "Fiery ale".to_string(), heal: 2, atk_bonus: 2, def_bonus: 0 },
            1 => Consumable { name: "Weeping Willow bark".to_string(), heal: 3, atk_bonus: 0, def_bonus: 0 },
            2 => Consumable { name: "Sunny Jerky".to_string(), heal: 5, atk_bonus: 0, def_bonus: 0 },
            _ => Consumable { name: "Frozen tears".to_string(), heal: -2, atk_bonus: 0, def_bonus: 5 },
        }
    }

    fn scatter_chests(map: &mut Map, seed: u64, spawn: (i32, i32), door: (i32, i32)) -> Vec<Chest> {
        let mut floors = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) == Tile::Floor { floors.push((x as i32, y as i32)); }
            }
        }
        let mut rng = StdRng::seed_from_u64(seed);
        let mut chests = Vec::new();
        let mut exclude = vec![spawn, door];
        let count = 3usize.min(floors.len());
        for _ in 0..count {
            let mut pos = spawn;
            for _tries in 0..200 {
                let candidate = floors[rng.gen_range(0..floors.len())];
                if !exclude.contains(&candidate) { pos = candidate; break; }
            }
            exclude.push(pos);
            map.set(pos.0 as usize, pos.1 as usize, Tile::Chest);
            chests.push(Chest { x: pos.0, y: pos.1, item: Some(Self::random_consumable(&mut rng)), weapon: None, opened: false });
        }
        chests
    }

    pub fn push_log(&mut self, msg: impl Into<String>) {
        self.logs.push_back(msg.into());
        while self.logs.len() > 6 { self.logs.pop_front(); }
    }

    fn toggle_room(&mut self) {
        let old_room = self.current;
        let new_room = if old_room == 0 { 1 } else { 0 };
        self.current = new_room;
        let door_pos = self.levels[new_room].door;
        let map = &self.levels[new_room].map;
        let mut spawn = door_pos; 
        'search: for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = door_pos.0 + dx;
                let ny = door_pos.1 + dy;
                if map.in_bounds(nx, ny) && map.get(nx as usize, ny as usize) == Tile::Floor {
                    spawn = (nx, ny);
                    break 'search;
                }
            }
        }
        self.player.x = spawn.0;
        self.player.y = spawn.1;
        if new_room == 1 { self.push_log("You step through the door into Room 2...".to_string()); } 
        else { self.push_log("You step back into Room 1...".to_string()); }
    }

    fn toggle_inventory(&mut self) {
        self.inventory_open = !self.inventory_open;
        if self.inventory_open { self.stats_open = false; self.push_log("Inventory opened.".to_string()); } 
        else { self.push_log("Inventory closed.".to_string()); }
    }

    fn toggle_stats(&mut self) {
        self.stats_open = !self.stats_open;
        if self.stats_open { self.inventory_open = false; self.push_log("Stats opened.".to_string()); } 
        else { self.push_log("Stats closed.".to_string()); }
    }

    fn toggle_inventory_tab(&mut self) {
        let tab_before = self.player.inventory.tab;
        self.player.inventory.toggle_tab();
        let tab_after = self.player.inventory.tab;
        let name = match tab_after { InvTab::Weapons => "Weapons", InvTab::Consumables => "Consumables", InvTab::Backpack => "Backpack" };
        if tab_before != tab_after { self.push_log(format!("Inventory tab: {}", name)); }
    }

    fn use_or_unequip_or_equip(&mut self) {
        let selection = self.player.inventory.selection();
        let mut log_msg: Option<String> = None;

        match selection {
            InvSelection::SwordSlot => {
                let eq_opt = self.player.inventory.sword.take();
                if let Some(eq) = eq_opt {
                    self.player.max_hp -= eq.hp_bonus; // Remove HP bonus
                    if self.player.hp > self.player.max_hp { self.player.hp = self.player.max_hp; }
                    self.player.inventory.backpack.push(eq.clone());
                    log_msg = Some(format!("Unequipped {}.", eq.name));
                } else { log_msg = Some("No sword equipped.".to_string()); }
            }
            InvSelection::ShieldSlot => {
                let eq_opt = self.player.inventory.shield.take();
                if let Some(eq) = eq_opt {
                    self.player.max_hp -= eq.hp_bonus; // Remove HP bonus
                    if self.player.hp > self.player.max_hp { self.player.hp = self.player.max_hp; }
                    self.player.inventory.backpack.push(eq.clone());
                    log_msg = Some(format!("Unequipped {}.", eq.name));
                } else { log_msg = Some("No shield equipped.".to_string()); }
            }
            InvSelection::Consumable(_) => {
                let item_opt = self.player.inventory.take_selected_consumable();
                if let Some(item) = item_opt {
                    let before = self.player.hp;
                    self.player.hp = (self.player.hp + item.heal).min(self.player.max_hp);
                    let healed = self.player.hp - before;
                    if item.atk_bonus != 0 || item.def_bonus != 0 {
                        self.player.add_temp_buff(item.atk_bonus, item.def_bonus, 0, Duration::from_secs(30));
                    }
                    let mut effects = vec![Self::fmt_hp_delta(healed)];
                    let fmt_signed = |v: i32| if v >= 0 { format!("+{}", v) } else { format!("{}", v) };
                    if item.atk_bonus != 0 { effects.push(format!("{} ATK/30sec", fmt_signed(item.atk_bonus))); }
                    if item.def_bonus != 0 { effects.push(format!("{} DEF/30sec", fmt_signed(item.def_bonus))); }
                    log_msg = Some(format!("Used {} ({}).", item.name, effects.join(", ")));
                } else { log_msg = Some("No consumables to use.".to_string()); }
            }
            InvSelection::BackpackItem(i) => {
                let eq_opt = if i < self.player.inventory.backpack.len() { Some(self.player.inventory.backpack.remove(i)) } else { None };
                if let Some(eq) = eq_opt {
                    // Add new HP bonus
                    self.player.max_hp += eq.hp_bonus;
                    
                    match eq.slot {
                        Slot::Sword => {
                            if let Some(old) = self.player.inventory.sword.take() { 
                                self.player.max_hp -= old.hp_bonus; // Remove old bonus
                                self.player.inventory.backpack.push(old); 
                            }
                            self.player.inventory.sword = Some(eq.clone());
                            log_msg = Some(format!("Equipped sword: {}.", eq.name));
                        }
                        Slot::Shield => {
                            if let Some(old) = self.player.inventory.shield.take() { 
                                self.player.max_hp -= old.hp_bonus; // Remove old bonus
                                self.player.inventory.backpack.push(old); 
                            }
                            self.player.inventory.shield = Some(eq.clone());
                            log_msg = Some(format!("Equipped shield: {}.", eq.name));
                        }
                    }
                    // Clamp HP if max reduced
                    if self.player.hp > self.player.max_hp { self.player.hp = self.player.max_hp; }
                    
                    let inv = &mut self.player.inventory;
                    if inv.backpack.is_empty() { inv.backpack_cursor = 0; } else if inv.backpack_cursor >= inv.backpack.len() { inv.backpack_cursor = inv.backpack.len() - 1; }
                } else { log_msg = Some("Nothing to equip.".to_string()); }
            }
            InvSelection::None => { log_msg = Some("Nothing to use.".to_string()); }
        }
        if let Some(m) = log_msg { self.push_log(m); }
    }

    fn start_chest_dialogue(&mut self, room: usize, x: i32, y: i32, item: Option<Consumable>, weapon: Option<Equipment>) {
        let name = if let Some(c) = &item { c.name.clone() } else if let Some(w) = &weapon { w.name.clone() } else { "nothing".to_string() };
        let pages = vec![format!(
            "You found a treasure chest!\nInside is: {}\n\n(A) Put in inventory\n(B) Use now (Consumable)\n(C) Throw away",
            name
        )];
        self.dialogue = Some(DialogueSession {
            npc: NpcId::MayorSol, title: "Treasure Chest".to_string(), pages, page_index: 0,
            awaiting: Some(AwaitingChoice::Chest { room, x, y, item, weapon }),
        });
        self.state = GameState::Dialogue;
    }

    fn open_chest_if_on_one(&mut self) {
        let room = self.current;
        let px = self.player.x;
        let py = self.player.y;
        let level = &mut self.levels[room];
        if let Some(chest) = level.chests.iter_mut().find(|c| !c.opened && c.x == px && c.y == py) {
            chest.opened = true;
            level.map.set(px as usize, py as usize, Tile::Floor);
            let item = chest.item.take();
            let weapon = chest.weapon.take();
            self.start_chest_dialogue(room, px, py, item, weapon);
        }
    }

    // --- BATTLE LOGIC ---
    fn start_battle(&mut self, enemy_id: NpcId) {
        let (name, hp, atk, def, spd) = match enemy_id {
            NpcId::Shab => ("Shab", 10, 3, 0, 4),
            NpcId::Krad => ("Krad", 20, 6, 4, 0),
            NpcId::Mah => ("Mah", 30, 12, 10, 8),
            _ => return,
        };

        self.battle = Some(BattleSession {
            enemy_id,
            enemy_name: name.to_string(),
            enemy_hp: hp,
            enemy_max_hp: hp,
            enemy_atk: atk,
            enemy_def: def,
            enemy_speed: spd,
            penalty_mode: false,
            player_initiated: false, 
        });
        self.state = GameState::Battle;
        self.push_log(format!("Battle started against {}!", name));
    }

    fn calc_damage(atk: i32) -> i32 {
        (atk as f32 * 1.2) as i32
    }

    fn try_deflect(def: i32) -> bool {
        let chance = (def as f32 / 10.0) * 0.2;
        rand::random::<f32>() < chance
    }

    fn apply_battle_turn(&mut self, opt: u8, penalty: bool) {
        let mut end_battle = false;
        let mut player_won = false;

        if let Some(mut bs) = self.battle.take() {
            if penalty { bs.penalty_mode = true; }
            let p_spd = self.player.speed();
            let e_spd = bs.enemy_speed;
            let player_first = if bs.penalty_mode { false } else { p_spd >= e_spd };

            match opt {
                1 => { // Fight
                    if player_first {
                        self.perform_player_attack(&mut bs);
                        if bs.enemy_hp > 0 { self.perform_enemy_attack(&mut bs); }
                    } else {
                        self.perform_enemy_attack(&mut bs);
                        if self.player.hp > 0 { self.perform_player_attack(&mut bs); }
                    }
                }
                2 => { // Inventory Used
                    self.perform_enemy_attack(&mut bs);
                }
                3 => { // Run
                    if bs.player_initiated {
                        self.push_log("You started this, finish it!");
                        self.perform_enemy_attack(&mut bs);
                    } else {
                        if rand::random::<f32>() < 0.5 {
                            self.push_log("You fled the battle!");
                            end_battle = true;
                        } else {
                            self.push_log("Failed to flee!");
                            self.perform_enemy_attack(&mut bs);
                        }
                    }
                }
                _ => {}
            }

            if bs.enemy_hp <= 0 {
                player_won = true;
                end_battle = true;
            }
            if self.player.hp <= 0 {
                end_battle = true;
            }

            if !end_battle {
                self.battle = Some(bs);
            } else {
                if player_won {
                    self.handle_win(bs.enemy_id);
                }
                self.state = GameState::Playing;
            }
        }
    }

    fn perform_player_attack(&mut self, bs: &mut BattleSession) {
        let dmg = Self::calc_damage(self.player.attack());
        if Self::try_deflect(bs.enemy_def) {
            self.push_log(format!("{} deflected your attack!", bs.enemy_name));
        } else {
            bs.enemy_hp -= dmg;
            self.push_log(format!("You hit {} for {} dmg.", bs.enemy_name, dmg));
        }
    }

    fn perform_enemy_attack(&mut self, bs: &mut BattleSession) {
        let dmg = Self::calc_damage(bs.enemy_atk);
        if Self::try_deflect(self.player.defense()) {
            self.push_log(format!("You deflected {}'s attack!", bs.enemy_name));
        } else {
            self.player.hp -= dmg;
            self.push_log(format!("{} hit you for {} dmg.", bs.enemy_name, dmg));
        }
    }

    fn handle_win(&mut self, id: NpcId) {
        match id {
            NpcId::Shab => {
                self.shab_defeated = true;
                self.start_dialogue_raw("Shab", vec!["I can’t believe I lost to the likes of you…".to_string()]);
            }
            NpcId::Krad => {
                self.krad_defeated = true;
                self.start_dialogue_raw("Krad", vec!["My armor….".to_string()]);
            }
            NpcId::Mah => {
                self.mah_defeated = true;
                
                // Remove boss and spawn Weeping Dagger Chest
                let boss_pos = if let Some(pos) = self.npcs.iter().position(|n| n.id == NpcId::Mah) {
                    let npc = self.npcs.remove(pos);
                    let chest = Chest {
                        x: npc.x, y: npc.y,
                        item: None,
                        weapon: Some(Equipment {
                            name: "Weeping Dagger".to_string(),
                            slot: Slot::Sword,
                            hp_bonus: -100, atk_bonus: -100, def_bonus: -100, speed_bonus: -100,
                        }),
                        opened: false
                    };
                    self.levels[1].chests.push(chest);
                    self.levels[1].map.set(npc.x as usize, npc.y as usize, Tile::Chest);
                    Some((npc.x, npc.y))
                } else { None };

                // Spawn Shield of Healing Chest (Random location in Room 2, away from dagger)
                if let Some((bx, by)) = boss_pos {
                    // Create taken list including boss loc, door, existing chests
                    let mut taken = vec![(bx, by), self.levels[1].door];
                    for c in &self.levels[1].chests { taken.push((c.x, c.y)); }
                    
                    // Find spot with dist >= 10 from Dagger chest
                    let (sx, sy) = self.random_floor_spaced(1, &taken, 10);
                    
                    let shield_chest = Chest {
                        x: sx, y: sy,
                        item: None,
                        weapon: Some(Equipment {
                            name: "Shield of healing".to_string(),
                            slot: Slot::Shield,
                            hp_bonus: 2, atk_bonus: 0, def_bonus: 10, speed_bonus: 0,
                        }),
                        opened: false
                    };
                    self.levels[1].chests.push(shield_chest);
                    self.levels[1].map.set(sx as usize, sy as usize, Tile::Chest);
                }

                self.start_dialogue_raw("Mah", vec![
                    "I underestimated you…".to_string(),
                    "Listen, Sol, is not…".to_string(), "what".to_string(), "you".to_string(), "thin-".to_string()
                ]);
            }
            _ => {}
        }
        self.push_log("You won the battle!");
    }

    fn start_dialogue_raw(&mut self, title: &str, pages: Vec<String>) {
        self.dialogue = Some(DialogueSession {
            npc: NpcId::Random1,
            title: title.to_string(),
            pages,
            page_index: 0,
            awaiting: None,
        });
        self.state = GameState::Dialogue;
    }

    fn start_dialogue_for(&mut self, npc: &Npc) {
        let session = match npc.id {
            // Existing NPCs
            NpcId::MayorSol => {
                 if self.mayor_done { DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Well, what’re you still standing here for? GO TO NOOR!".to_string()], page_index: 0, awaiting: None } } 
                 else { DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Welcome to Sunny Days, visitor! I am Mayor Sol. We are normally much more able to take in tourists, but you may have arrived at a bad time. The Weeping have made it a rough time, they have completely taken over the Weeping Willow forests.".to_string(), "What’s that? The weeping sound like they belong in the Weeping Willow Forests? No! That’s nonsense, the only reason they are called the weeping, is because they WEEP before they kill! I mean, is it not right there in the name? Keep up! Ok, but my friend, you MUST help us get them out. Without our Weeping Willow bark, we are losing our health! Please will you help? (Y/N)".to_string()], page_index: 0, awaiting: Some(AwaitingChoice::YesNoMayor) } }
            },
            NpcId::Noor => {
                 if self.noor_done { DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Scram! Go to Lamp and get whatever you’re missing!!".to_string()], page_index: 0, awaiting: None } }
                 else { DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Hey there partner!".to_string(), "What’s that, the Mayor sent you here? Damn Sol, always ruining my day. What! No not you, you seem okay… ish. So you’re gonna go and fight the Weeping ay? Well you’ll need a weapon. Grab one: (A) Basic Sword  (B) Basic Shield".to_string(), "Good choice! Now I’ll keep the other one to be fair, if you want your second choice, go see Lamp!".to_string()], page_index: 0, awaiting: Some(AwaitingChoice::ABNoorWeapon) } }
            },
            NpcId::Lamp => {
                 if !self.noor_done { DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Hey aren’t you supposed to talk to Noor first?".to_string()], page_index: 0, awaiting: None } }
                 else if self.lamp_done { DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Well good luck, if you’re fighting the Weeping, you’ll need it!".to_string()], page_index: 0, awaiting: None } }
                 else {
                    let missing = if self.player.inventory.sword.is_none() { "Sword" } else { "Shield" };
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec![format!("Hey! Did Noor send you? Yeah, they’re a bit rough around the edges. So you’re missing a {}, well take this!", missing), format!("You got the {}.", missing)], page_index: 0, awaiting: None }
                 }
            },
            NpcId::Random1 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Isn’t it bad? So gloomy, so dark, I need some vitamin D pills or something!".to_string()], page_index: 0, awaiting: None },
            NpcId::Random2 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["I actually overheard the Mayor talking to himself, I think he’s going a bit cukoo!!".to_string()], page_index: 0, awaiting: None },
            NpcId::Random3 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Oh please, if you think the Weeping are bad, wait until you hear from the IRS!".to_string()], page_index: 0, awaiting: None },
            
            NpcId::Weeping1 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["I can’t believe that’s how they think of us in here, we literally get our name from the Weeping Willow trees that we LIVE in. Like come on!".to_string()], page_index: 0, awaiting: None },
            NpcId::Weeping2 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["It sure is cold out, all that global warming bibble babble is a hoax!".to_string()], page_index: 0, awaiting: None },
            NpcId::Weeping3 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Have you talked to the guy who thinks global warming is fake? What a nut!".to_string()], page_index: 0, awaiting: None },
            NpcId::Weeping4 => DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["I had a friend in that village…".to_string(), "His name meant bright, just like how he was.".to_string(), "I wonder how he’s doing…".to_string()], page_index: 0, awaiting: None },

            NpcId::Shab => {
                if self.shab_defeated {
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Get away from me, I’m training…".to_string()], page_index: 0, awaiting: None }
                } else {
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Hey! You’re not supposed to be in here, who are you?!".to_string(), "Wait, nevermind, I couldn’t care less, are you ready to die?!!!".to_string()], page_index: 0, awaiting: None }
                }
            },
            NpcId::Krad => {
                if self.krad_defeated {
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["W-what do you want from me?!?!?!".to_string(), "LEAVE ME BE, you’ve shattered my honor, and my armor….".to_string(), " *sniffles* ".to_string()], page_index: 0, awaiting: None }
                } else {
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Who are you…".to_string(), "Doesn’t matter… my armor…".to_string(), "IS IMPENETRABLE".to_string()], page_index: 0, awaiting: None }
                }
            },
            NpcId::Mah => {
                if self.shab_defeated && self.krad_defeated {
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["You low-class scum".to_string(), "You come into my home, my community…".to_string(), "AND KILL MY MEN?!?!?!".to_string()], page_index: 0, awaiting: None }
                } else {
                    DialogueSession { npc: npc.id, title: npc.name.clone(), pages: vec!["Insignificant being, begone from my sight, my men will handle you…".to_string()], page_index: 0, awaiting: None }
                }
            }
        };
        self.dialogue = Some(session);
        self.state = GameState::Dialogue;
    }

    fn dialogue_continue(&mut self) {
        let mut start_battle_id = None;
        if let Some(d) = &mut self.dialogue {
            if d.page_index + 1 < d.pages.len() {
                d.page_index += 1;
            } else {
                match d.npc {
                    NpcId::Shab if !self.shab_defeated => start_battle_id = Some(NpcId::Shab),
                    NpcId::Krad if !self.krad_defeated => start_battle_id = Some(NpcId::Krad),
                    NpcId::Mah if !self.mah_defeated && self.shab_defeated && self.krad_defeated => start_battle_id = Some(NpcId::Mah),
                    _ => {}
                }
                self.dialogue = None;
                self.state = GameState::Playing;
            }
        }
        if let Some(id) = start_battle_id {
            self.start_battle(id);
        }
    }

    fn dialogue_choice(&mut self, c: char) {
        let awaiting = self.dialogue.as_ref().and_then(|d| d.awaiting.clone());
        let up = c.to_ascii_uppercase();

        match awaiting {
            Some(AwaitingChoice::YesNoMayor) => {
                if up == 'Y' || up == 'N' {
                    let yes = up == 'Y';
                    self.mayor_done = true;
                    if let Some(d) = &mut self.dialogue {
                        d.awaiting = None;
                        d.pages = vec![if yes { "Why thank you! Now go talk to Noor to get you started.".to_string() } else { "Aren’t you rude, I’ve been nothing but kind. Fine, go to Noor to get you started I guess…".to_string() }];
                        d.page_index = 0;
                    }
                }
            }
            Some(AwaitingChoice::ABNoorWeapon) => {
                if up == 'A' || up == 'B' {
                    if up == 'A' { self.player.equip_sword(Equipment { name: "Basic Sword".to_string(), slot: Slot::Sword, hp_bonus: 0, atk_bonus: 3, def_bonus: 0, speed_bonus: 3 }); } 
                    else { self.player.equip_shield(Equipment { name: "Basic Shield".to_string(), slot: Slot::Shield, hp_bonus: 0, atk_bonus: 0, def_bonus: 3, speed_bonus: -2 }); }
                    self.noor_done = true;
                    if let Some(d) = &mut self.dialogue { d.awaiting = None; d.page_index = 2; }
                }
            }
            Some(AwaitingChoice::Chest { item, weapon, .. }) => {
                let mut log = None;
                match up {
                    'A' => {
                        if let Some(w) = weapon {
                            self.player.inventory.backpack.push(w.clone());
                            log = Some(format!("Picked up {}.", w.name));
                        } else if let Some(cons) = item {
                            if self.player.inventory.consumables.len() < 10 {
                                self.player.inventory.consumables.push(cons.clone());
                                log = Some(format!("Picked up {}.", cons.name));
                            } else { log = Some("Slots full.".to_string()); }
                        }
                    }
                    'B' => {
                        if let Some(cons) = item {
                            let before = self.player.hp;
                            self.player.hp = (self.player.hp + cons.heal).min(self.player.max_hp);
                            let healed = self.player.hp - before;
                            log = Some(format!("Used {} ({}).", cons.name, Self::fmt_hp_delta(healed)));
                        } else { log = Some("Cannot use that.".to_string()); }
                    }
                    'C' => { log = Some("Left chest.".to_string()); }
                    _ => return,
                }
                if let Some(m) = log { self.push_log(m); }
                self.dialogue = None;
                self.state = GameState::Playing;
            }
            None => {}
        }
    }

    fn door_near_player(&self) -> Option<(i32, i32)> {
        let px = self.player.x;
        let py = self.player.y;
        let map = self.current_map();
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = px + dx;
                let ny = py + dy;
                if map.in_bounds(nx, ny) && map.get(nx as usize, ny as usize) == Tile::Door {
                    return Some((nx, ny));
                }
            }
        }
        None
    }

    pub fn apply_action(&mut self, action: Action) -> bool {
        self.player.purge_expired_buffs();
        match self.state {
            GameState::Title => match action { Action::Confirm => self.state = GameState::Intro, Action::Quit => return false, _ => {} },
            GameState::Intro => match action { Action::Confirm => self.state = GameState::Playing, Action::Quit => return false, _ => {} },
            GameState::Dialogue => match action { Action::Confirm => self.dialogue_continue(), Action::Choice(c) => self.dialogue_choice(c), Action::Quit => return false, _ => {} },
            
            GameState::Battle => match action {
                Action::BattleOption(opt, penalty) => {
                    if opt == 1 || opt == 3 {
                        if opt == 1 && !penalty && self.battle.as_ref().map_or(false, |b| b.enemy_speed < self.player.speed()) {
                            if let Some(bs) = &mut self.battle { bs.player_initiated = true; }
                        }
                        self.apply_battle_turn(opt, penalty);
                    } else if opt == 2 {
                        self.inventory_open = true;
                        self.player.inventory.tab = InvTab::Consumables;
                    }
                }
                Action::UseConsumable => {
                    if self.inventory_open {
                        self.use_or_unequip_or_equip();
                        self.inventory_open = false;
                        self.apply_battle_turn(2, false);
                    }
                }
                Action::ToggleInventory | Action::Quit => {
                    if self.inventory_open { self.inventory_open = false; }
                    else if matches!(action, Action::Quit) { return false; }
                }
                Action::InventoryUp => { if self.inventory_open { self.player.inventory.move_cursor(-1); } }
                Action::InventoryDown => { if self.inventory_open { self.player.inventory.move_cursor(1); } }
                _ => {}
            }

            GameState::Playing => match action {
                Action::ToggleStats => self.toggle_stats(),
                Action::ToggleInventory => self.toggle_inventory(),
                Action::ToggleInvTab => if self.inventory_open { self.toggle_inventory_tab() },
                Action::InventoryUp => if self.inventory_open { self.player.inventory.move_cursor(-1) },
                Action::InventoryDown => if self.inventory_open { self.player.inventory.move_cursor(1) },
                Action::UseConsumable => if self.inventory_open { self.use_or_unequip_or_equip() },
                Action::Interact => {
                    if let Some(npc) = self.npc_near_player().cloned() {
                        self.start_dialogue_for(&npc);
                        if self.noor_done && npc.id == NpcId::Lamp && !self.lamp_done {
                            let ms = self.player.inventory.sword.is_none();
                            let msh = self.player.inventory.shield.is_none();
                            if ms { self.player.equip_sword(Equipment { name: "Basic Sword".to_string(), slot: Slot::Sword, hp_bonus: 0, atk_bonus: 3, def_bonus: 0, speed_bonus: 3 }); self.lamp_done = true; }
                            else if msh { self.player.equip_shield(Equipment { name: "Basic Shield".to_string(), slot: Slot::Shield, hp_bonus: 0, atk_bonus: 0, def_bonus: 3, speed_bonus: -2 }); self.lamp_done = true; }
                        }
                    } else {
                        if let Some(_) = self.door_near_player() {
                             if self.player.inventory.sword.is_some() && self.player.inventory.shield.is_some() { self.toggle_room(); } 
                             else { self.push_log("Talk to the mayor and come back"); }
                        } else {
                             self.open_chest_if_on_one();
                             if self.state != GameState::Dialogue { self.push_log("No one nearby."); }
                        }
                    }
                }
                Action::Move(dx, dy) => {
                    if self.inventory_open || self.stats_open { return true; }
                    let nx = self.player.x + dx;
                    let ny = self.player.y + dy;
                    if self.npc_at(self.current, nx, ny).is_some() { return true; }
                    let map_snap = self.current_map().clone();
                    self.player.try_move(dx, dy, &map_snap);
                    let newp = (self.player.x, self.player.y);
                    if self.current_map().get(newp.0 as usize, newp.1 as usize) == Tile::Chest { self.open_chest_if_on_one(); }
                }
                Action::Quit => return false,
                _ => {}
            },
        }
        true
    }
}