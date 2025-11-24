use crate::engine::action::Action;
use crate::engine::entity::{Equipment, Player, InvSelection, InvTab, EquipSlot};
use crate::map::{generator::generate_rooms_and_corridors, tile::Tile, Map};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

use std::collections::VecDeque;

#[derive(Clone)]
pub struct Level {
    pub map: Map,
    pub door: (i32, i32),
}

#[derive(Debug, Clone)]
pub enum GameState {
    Title,
    Intro,
    Playing,
    Dialogue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcId {
    MayorSol,
    Noor,
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
}

#[derive(Debug, Clone)]
pub struct DialogueSession {
    pub npc: NpcId,
    pub title: String,
    pub pages: Vec<String>,
    pub page_index: usize,
    pub awaiting: Option<AwaitingChoice>,
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

    pub dialogue: Option<DialogueSession>,
}

impl World {
    pub fn new(seed: u64, width: usize, height: usize) -> Self {
        let (level0, spawn0) = Self::make_level(seed, 0, width, height);
        let (level1, _spawn1) = Self::make_level(seed, 1, width, height);

        let mut logs = VecDeque::new();
        logs.push_back(format!("Seed: {}", seed));
        logs.push_back("Welcome to Sunny Day(s).".to_string());
        logs.push_back("Move with WASD or arrow keys.".to_string());
        logs.push_back("Press E to talk to NPCs.".to_string());
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
            dialogue: None,
        };

        world.spawn_npcs(spawn0);
        world
    }

    fn spawn_npcs(&mut self, spawn0: (i32, i32)) {
        let mut mx = spawn0.0 + 5;
        let mut my = spawn0.1;

        if !self.is_floor(0, mx, my) {
            let candidates = [
                (mx, my),
                (mx, my + 1),
                (mx, my - 1),
                (mx + 1, my),
                (mx - 1, my),
            ];
            for (cx, cy) in candidates {
                if self.is_floor(0, cx, cy) {
                    mx = cx; my = cy;
                    break;
                }
            }
        }

        self.npcs.push(Npc {
            id: NpcId::MayorSol,
            name: "Mayor Sol".to_string(),
            room: 0,
            x: mx,
            y: my,
            symbol: 'M',
        });

        let (nx, ny) = self.random_floor_excluding(0, &[(spawn0.0, spawn0.1), (mx, my)]);
        self.npcs.push(Npc {
            id: NpcId::Noor,
            name: "Noor".to_string(),
            room: 0,
            x: nx,
            y: ny,
            symbol: 'N',
        });
    }

    fn is_floor(&self, room: usize, x: i32, y: i32) -> bool {
        let map = &self.levels[room].map;
        if x < 0 || y < 0 || x >= map.width as i32 || y >= map.height as i32 {
            return false;
        }
        map.get(x as usize, y as usize) == Tile::Floor
    }

    fn random_floor_excluding(&self, room: usize, exclude: &[(i32, i32)]) -> (i32, i32) {
        let map = &self.levels[room].map;
        let mut floors = Vec::new();
        for y in 0..map.height {
            for x in 0..map.width {
                if map.get(x, y) == Tile::Floor {
                    let p = (x as i32, y as i32);
                    if !exclude.contains(&p) {
                        floors.push(p);
                    }
                }
            }
        }
        let mut rng = StdRng::seed_from_u64(self.seed ^ 0xBEEFu64);
        floors[rng.gen_range(0..floors.len())]
    }

    pub fn intro_lines(&self) -> &[String] {
        &self.intro_lines
    }

    fn current_level(&self) -> &Level {
        &self.levels[self.current]
    }

    pub fn current_map(&self) -> &Map {
        &self.current_level().map
    }

    pub fn npc_at(&self, room: usize, x: i32, y: i32) -> Option<&Npc> {
        self.npcs.iter().find(|n| n.room == room && n.x == x && n.y == y)
    }

    pub fn npc_near_player(&self) -> Option<&Npc> {
        let px = self.player.x;
        let py = self.player.y;
        self.npcs.iter().find(|n| {
            n.room == self.current &&
            (n.x - px).abs().max((n.y - py).abs()) <= 1
        })
    }

    fn make_level(base_seed: u64, depth: usize, width: usize, height: usize) -> (Level, (i32, i32)) {
        let seed = base_seed.wrapping_add(depth as u64 * 9_973);
        let mut map = generate_rooms_and_corridors(width, height, seed);

        let (sx, sy) = map.find_first_floor().unwrap_or((1, 1));
        let spawn = (sx as i32, sy as i32);

        let door = Self::place_random_door(&mut map, seed ^ 0xD00D, spawn);

        (Level { map, door }, spawn)
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

        let target = self.levels[new_room].door;
        self.player.x = target.0;
        self.player.y = target.1;

        if new_room == 1 {
            self.push_log("You step through the door into Room 2...".to_string());
        } else {
            self.push_log("You step back into Room 1...".to_string());
        }
    }

    fn toggle_inventory(&mut self) {
        self.inventory_open = !self.inventory_open;
        if self.inventory_open {
            self.stats_open = false;
            self.push_log("Inventory opened.".to_string());
        } else {
            self.push_log("Inventory closed.".to_string());
        }
    }

    fn toggle_stats(&mut self) {
        self.stats_open = !self.stats_open;
        if self.stats_open {
            self.inventory_open = false;
            self.push_log("Stats opened.".to_string());
        } else {
            self.push_log("Stats closed.".to_string());
        }
    }

    fn toggle_inventory_tab(&mut self) {
        let tab_before = self.player.inventory.tab;
        self.player.inventory.toggle_tab();
        let tab_after = self.player.inventory.tab;

        let name = match tab_after {
            InvTab::Weapons => "Weapons",
            InvTab::Consumables => "Consumables",
            InvTab::Backpack => "Backpack",
        };

        if tab_before != tab_after {
            self.push_log(format!("Inventory tab: {}", name));
        }
    }

    // ✅ FIXED: no long-lived &mut borrow while logging
    fn use_or_unequip_or_equip(&mut self) {
        let selection = self.player.inventory.selection();
        let mut log_msg: Option<String> = None;

        match selection {
            InvSelection::SwordSlot => {
                let eq_opt = self.player.inventory.sword.take();
                if let Some(eq) = eq_opt {
                    self.player.inventory.backpack.push(eq.clone());
                    log_msg = Some(format!("Unequipped {}.", eq.name));
                } else {
                    log_msg = Some("No sword equipped.".to_string());
                }
            }

            InvSelection::ShieldSlot => {
                let eq_opt = self.player.inventory.shield.take();
                if let Some(eq) = eq_opt {
                    self.player.inventory.backpack.push(eq.clone());
                    log_msg = Some(format!("Unequipped {}.", eq.name));
                } else {
                    log_msg = Some("No shield equipped.".to_string());
                }
            }

            InvSelection::Consumable(_) => {
                let item_opt = self.player.inventory.take_selected_consumable();
                if let Some(item) = item_opt {
                    let before = self.player.hp;
                    self.player.hp = (self.player.hp + item.heal).min(self.player.max_hp);
                    let healed = self.player.hp - before;
                    log_msg = Some(format!("Used {} (+{} HP).", item.name, healed));
                } else {
                    log_msg = Some("No consumables to use.".to_string());
                }
            }

            InvSelection::BackpackItem(i) => {
                let eq_opt = {
                    let inv = &mut self.player.inventory;
                    if i >= inv.backpack.len() {
                        None
                    } else {
                        Some(inv.backpack.remove(i))
                    }
                };

                if let Some(eq) = eq_opt {
                    match eq.slot {
                        EquipSlot::Sword => {
                            if let Some(old) = self.player.inventory.sword.take() {
                                self.player.inventory.backpack.push(old);
                            }
                            self.player.inventory.sword = Some(eq.clone());
                            log_msg = Some(format!("Equipped sword: {}.", eq.name));
                        }
                        EquipSlot::Shield => {
                            if let Some(old) = self.player.inventory.shield.take() {
                                self.player.inventory.backpack.push(old);
                            }
                            self.player.inventory.shield = Some(eq.clone());
                            log_msg = Some(format!("Equipped shield: {}.", eq.name));
                        }
                    }

                    // clamp backpack cursor after removal
                    let inv = &mut self.player.inventory;
                    if inv.backpack.is_empty() {
                        inv.backpack_cursor = 0;
                    } else if inv.backpack_cursor >= inv.backpack.len() {
                        inv.backpack_cursor = inv.backpack.len() - 1;
                    }
                } else {
                    log_msg = Some("Nothing to equip.".to_string());
                }
            }

            InvSelection::None => {
                log_msg = Some("Nothing to use.".to_string());
            }
        }

        if let Some(m) = log_msg {
            self.push_log(m);
        }
    }

    fn start_dialogue_for(&mut self, npc: &Npc) {
        let session = match npc.id {
            NpcId::MayorSol => {
                if self.mayor_done {
                    DialogueSession {
                        npc: npc.id,
                        title: npc.name.clone(),
                        pages: vec![
                            "Well, what’re you still standing here for? GO TO NOOR!".to_string()
                        ],
                        page_index: 0,
                        awaiting: None,
                    }
                } else {
                    DialogueSession {
                        npc: npc.id,
                        title: npc.name.clone(),
                        pages: vec![
                            "Welcome to Sunny Days, visitor! I am Mayor Sol. We are normally much more able to take in tourists, but you may have arrived at a bad time. The Weeping have made it a rough time, they have completely taken over the Weeping Willow forests.".to_string(),
                            "What’s that? The weeping sound like they belong in the Weeping Willow Forests? No! That’s nonsense, the only reason they are called the weeping, is because they WEEP before they kill! I mean, is it not right there in the name? Keep up! Ok, but my friend, you MUST help us get them out. Without our Weeping Willow bark, we are losing our health! Please will you help? (Y/N)".to_string(),
                        ],
                        page_index: 0,
                        awaiting: Some(AwaitingChoice::YesNoMayor),
                    }
                }
            }

            NpcId::Noor => {
                if self.noor_done {
                    DialogueSession {
                        npc: npc.id,
                        title: npc.name.clone(),
                        pages: vec![
                            "Good luck out there. Don’t let Sol boss you around too much.".to_string()
                        ],
                        page_index: 0,
                        awaiting: None,
                    }
                } else {
                    DialogueSession {
                        npc: npc.id,
                        title: npc.name.clone(),
                        pages: vec![
                            "Hey there partner!".to_string(),
                            "What’s that, the Mayor sent you here? Damn Sol, always ruining my day. What! No not you, you seem okay… ish. So you’re gonna go and fight the Weeping ay? Well you’ll need a weapon. Grab one: (A) Basic Sword  (B) Basic Shield".to_string(),
                            "Good choice! Now scram!".to_string(),
                        ],
                        page_index: 0,
                        awaiting: Some(AwaitingChoice::ABNoorWeapon),
                    }
                }
            }
        };

        self.dialogue = Some(session);
        self.state = GameState::Dialogue;
    }

    fn dialogue_continue(&mut self) {
        if let Some(d) = &mut self.dialogue {
            if d.page_index + 1 < d.pages.len() {
                d.page_index += 1;
            } else {
                if d.awaiting.is_none() {
                    self.dialogue = None;
                    self.state = GameState::Playing;
                }
            }
        }
    }

    fn dialogue_choice(&mut self, c: char) {
        let Some(d) = &mut self.dialogue else { return; };

        match d.awaiting {
            Some(AwaitingChoice::YesNoMayor) => {
                let up = c.to_ascii_uppercase();
                if up == 'Y' {
                    self.mayor_done = true;
                    d.awaiting = None;
                    d.pages = vec![
                        "Why thank you! Now go talk to Noor to get you started.".to_string()
                    ];
                    d.page_index = 0;
                } else if up == 'N' {
                    self.mayor_done = true;
                    d.awaiting = None;
                    d.pages = vec![
                        "Aren’t you rude, I’ve been nothing but kind. Fine, go to Noor to get you started I guess…".to_string()
                    ];
                    d.page_index = 0;
                }
            }

            Some(AwaitingChoice::ABNoorWeapon) => {
                let up = c.to_ascii_uppercase();
                if up == 'A' {
                    self.player.equip_sword(Equipment {
                        name: "Basic Sword".to_string(),
                        slot: EquipSlot::Sword,
                        atk_bonus: 3,
                        def_bonus: 0,
                        speed_bonus: 3,
                    });
                    self.noor_done = true;
                    d.awaiting = None;
                    d.page_index = 2;
                } else if up == 'B' {
                    self.player.equip_shield(Equipment {
                        name: "Basic Shield".to_string(),
                        slot: EquipSlot::Shield,
                        atk_bonus: 0,
                        def_bonus: 3,
                        speed_bonus: -2,
                    });
                    self.noor_done = true;
                    d.awaiting = None;
                    d.page_index = 2;
                }
            }

            None => {}
        }
    }

    pub fn apply_action(&mut self, action: Action) -> bool {
        match self.state {
            GameState::Title => {
                match action {
                    Action::Confirm => self.state = GameState::Intro,
                    Action::Quit => return false,
                    _ => {}
                }
                true
            }

            GameState::Intro => {
                match action {
                    Action::Confirm => self.state = GameState::Playing,
                    Action::Quit => return false,
                    _ => {}
                }
                true
            }

            GameState::Dialogue => {
                match action {
                    Action::Confirm => self.dialogue_continue(),
                    Action::Choice(c) => self.dialogue_choice(c),
                    Action::Quit => return false,
                    _ => {}
                }
                true
            }

            GameState::Playing => {
                match action {
                    Action::ToggleStats => {
                        self.toggle_stats();
                        true
                    }

                    Action::ToggleInventory => {
                        self.toggle_inventory();
                        true
                    }

                    Action::ToggleInvTab => {
                        if self.inventory_open {
                            self.toggle_inventory_tab();
                        }
                        true
                    }

                    Action::InventoryUp => {
                        if self.inventory_open {
                            self.player.inventory.move_cursor(-1);
                        }
                        true
                    }

                    Action::InventoryDown => {
                        if self.inventory_open {
                            self.player.inventory.move_cursor(1);
                        }
                        true
                    }

                    Action::UseConsumable => {
                        if self.inventory_open {
                            self.use_or_unequip_or_equip();
                        }
                        true
                    }

                    Action::Interact => {
                        if let Some(npc) = self.npc_near_player().cloned() {
                            self.start_dialogue_for(&npc);
                        } else {
                            self.push_log("No one nearby to talk to.".to_string());
                        }
                        true
                    }

                    Action::Move(dx, dy) => {
                        if self.inventory_open || self.stats_open {
                            return true;
                        }

                        let nx = self.player.x + dx;
                        let ny = self.player.y + dy;
                        if self.npc_at(self.current, nx, ny).is_some() {
                            return true;
                        }

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
                    Action::Confirm => true,
                    Action::Choice(_) => true,
                }
            }
        }
    }
}
