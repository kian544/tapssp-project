use crate::engine::world::{World, GameState, NpcId};
use crate::engine::entity::{InvTab, InvSelection};
use crate::map::tile::Tile;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
    Frame,
};

const ZOOM_W: i32 = 35;
const ZOOM_H: i32 = 20;

fn compute_viewport_origin(
    px: i32, py: i32,
    map_w: i32, map_h: i32,
    view_w: i32, view_h: i32,
) -> (i32, i32) {
    let mut x0 = px - view_w / 2;
    let mut y0 = py - view_h / 2;

    if x0 < 0 { x0 = 0; }
    if y0 < 0 { y0 = 0; }

    if view_w < map_w && x0 + view_w > map_w {
        x0 = map_w - view_w;
    }
    if view_h < map_h && y0 + view_h > map_h {
        y0 = map_h - view_h;
    }

    (x0, y0)
}

fn fmt_bonus(v: i32) -> String {
    if v >= 0 { format!("+{}", v) } else { format!("{}", v) }
}

pub fn render(f: &mut Frame, world: &World) {
    let size = f.size();
    f.render_widget(Clear, size);

    if size.width < 20 || size.height < 10 {
        let msg = Paragraph::new("Terminal too small — resize to play.")
            .block(Block::default().borders(Borders::ALL).title("Sunny Days"))
            .wrap(Wrap { trim: true });
        f.render_widget(msg, size);
        return;
    }

    match world.state {
        GameState::Title => draw_title(f, size),
        GameState::Intro => draw_intro_static(f, size, world),
        GameState::Playing | GameState::Dialogue => draw_playing(f, size, world),
        GameState::Battle => draw_battle(f, size, world),
        GameState::Fin => draw_fin(f, size),
    }
}

fn draw_title(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            "Sunny Day",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "By Kian Kakavandi",
            Style::default().fg(Color::White),
        )),
        Line::from(""),
        Line::from("Click space to continue"),
    ];

    let title = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(title, area);
}

fn draw_intro_static(f: &mut Frame, area: Rect, world: &World) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        "INTRO",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));
    for l in world.intro_lines() {
        lines.push(Line::from(l.clone()));
    }
    lines.push(Line::from(""));
    lines.push(Line::from("Click space to start"));

    let intro = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(intro, area);
}

fn draw_fin(f: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(
            "FIN",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "SUNNY DAY",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "BY KIAN KAKAVANDI",
            Style::default().fg(Color::White).add_modifier(Modifier::ITALIC),
        )),
        Line::from(""),
        Line::from("Press Ctrl+C to exit"),
    ];

    let fin = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    f.render_widget(fin, area);
}

fn draw_playing(f: &mut Frame, size: Rect, world: &World) {
    let log_h = (size.height / 4).clamp(5, 10);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(log_h),
        ])
        .split(size);

    let top = vertical[0];
    let bottom = vertical[1];

    let sidebar_w = (top.width / 3).clamp(20, 40);

    if top.width < sidebar_w + 25 {
        let stacked = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(12),
            ])
            .split(top);

        draw_map(f, stacked[0], world);
        draw_sidebar(f, stacked[1], world);
    } else {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(sidebar_w),
            ])
            .split(top);

        draw_map(f, horizontal[0], world);
        draw_sidebar(f, horizontal[1], world);
    }

    if world.dialogue.is_some() {
        draw_dialogue(f, bottom, world);
    } else if world.stats_open {
        draw_stats(f, bottom, world);
    } else {
        draw_logs(f, bottom, world);
    }
}

fn draw_battle(f: &mut Frame, size: Rect, world: &World) {
    let log_h = (size.height / 4).clamp(5, 10);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),
            Constraint::Length(log_h),
        ])
        .split(size);

    let top = vertical[0];
    let bottom = vertical[1];

    let sidebar_w = (top.width / 3).clamp(20, 40);

    if top.width < sidebar_w + 25 {
        let stacked = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),
                Constraint::Length(12),
            ])
            .split(top);

        draw_map(f, stacked[0], world);
        draw_sidebar(f, stacked[1], world);
    } else {
        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(sidebar_w),
            ])
            .split(top);

        draw_map(f, horizontal[0], world);
        draw_sidebar(f, horizontal[1], world);
    }

    if let Some(bs) = &world.battle {
        let mut lines = vec![
            Line::from(Span::styled(
                format!("BATTLE VS {}", bs.enemy_name),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            )),
            Line::from(format!("Enemy HP: {}/{}", bs.enemy_hp, bs.enemy_max_hp)),
            Line::from(""),
        ];
        
        if world.inventory_open {
             lines.push(Line::from("SELECT CONSUMABLE (Space) OR I to Cancel"));
             for (i, c) in world.player.inventory.consumables.iter().enumerate() {
                 let marker = if matches!(world.player.inventory.selection(), InvSelection::Consumable(idx) if idx == i) { ">" } else { " " };
                 lines.push(Line::from(format!("{} {}", marker, c.name)));
             }
        } else {
            lines.push(Line::from("1. Fight"));
            lines.push(Line::from("2. Inventory"));
            lines.push(Line::from("3. Run"));
        }
        
        lines.push(Line::from("--- Log ---"));
        for l in world.logs.iter().rev().take(3) {
            lines.push(Line::from(l.clone()));
        }

        let block = Block::default().borders(Borders::ALL).title("Battle").style(Style::default().fg(Color::Red));
        f.render_widget(Paragraph::new(lines).block(block).wrap(Wrap { trim: true }), bottom);
    }
}

fn draw_map(f: &mut Frame, area: Rect, world: &World) {
    f.render_widget(Clear, area);

    let map = world.current_map();
    let px = world.player.x;
    let py = world.player.y;

    let map_w = map.width as i32;
    let map_h = map.height as i32;

    let inner_w = (area.width as i32).saturating_sub(2);
    let inner_h = (area.height as i32).saturating_sub(2);

    let view_w = inner_w.max(1);
    let view_h = inner_h.max(1);

    let (x0, y0) = compute_viewport_origin(px, py, map_w, map_h, view_w, view_h);

    let zoom_w = ZOOM_W.min(view_w);
    let zoom_h = ZOOM_H.min(view_h);
    let half_zoom_w = zoom_w / 2;
    let half_zoom_h = zoom_h / 2;

    let mut lines: Vec<Line> = Vec::with_capacity(view_h as usize);

    for vy in 0..view_h {
        let wy = y0 + vy;
        let mut spans: Vec<Span> = Vec::with_capacity(view_w as usize);

        for vx in 0..view_w {
            let wx = x0 + vx;

            if (wx - px).abs() > half_zoom_w || (wy - py).abs() > half_zoom_h {
                spans.push(Span::raw(" "));
                continue;
            }

            if wx == px && wy == py {
                spans.push(Span::styled("@", Style::default().fg(Color::Yellow)));
                continue;
            }

            if let Some(npc) = world.npc_at(world.current, wx, wy) {
                let (style, bold) = match npc.id {
                    NpcId::MayorSol => (Style::default().fg(Color::Cyan), true),
                    NpcId::Noor => (Style::default().fg(Color::Magenta), true),
                    NpcId::Lamp | NpcId::Dorosht => (Style::default().fg(Color::Yellow), true),
                    NpcId::Random1 | NpcId::Random2 | NpcId::Random3 => {
                        (Style::default().fg(Color::Yellow), true)
                    }
                    NpcId::Weeping1 | NpcId::Weeping2 | NpcId::Weeping3 | NpcId::Weeping4 => {
                        (Style::default().fg(Color::LightBlue), true)
                    }
                    NpcId::Shab | NpcId::Krad | NpcId::Mah => {
                        (Style::default().fg(Color::Red), true)
                    }
                };
                spans.push(Span::styled(
                    npc.symbol.to_string(),
                    if bold { style.add_modifier(Modifier::BOLD) } else { style },
                ));
                continue;
            }

            if wx < 0 || wy < 0 || wx >= map_w || wy >= map_h {
                spans.push(Span::raw(" "));
                continue;
            }

            let tile = map.get(wx as usize, wy as usize);
            let (ch, style) = match tile {
                Tile::Wall => ("#", Style::default().fg(Color::DarkGray)),
                Tile::Floor => (" ", Style::default()),
                Tile::Door => ("+", Style::default().fg(Color::White)),
                Tile::Chest => ("C", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            };

            spans.push(Span::styled(ch, style));
        }

        lines.push(Line::from(spans));
    }

    let map_widget = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Map"))
        .wrap(Wrap { trim: false });

    f.render_widget(map_widget, area);
}

fn tab_label(tab: InvTab, active: InvTab, title: &str) -> Span<'static> {
    if tab == active {
        Span::styled(
            format!("[{}]", title),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" {} ", title),
            Style::default().fg(Color::DarkGray),
        )
    }
}

fn draw_sidebar(f: &mut Frame, area: Rect, world: &World) {
    f.render_widget(Clear, area);

    let p = &world.player;
    let inv = &p.inventory;
    let room_label = if world.current == 0 { "Room 1" } else { "Room 2" };

    let mut text: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("HP: ", Style::default().fg(Color::White)),
            Span::styled(
                format!("{}/{}", p.hp, p.max_hp),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(format!("ATK: {}", p.attack())),
        Line::from(format!("DEF: {}", p.defense())),
        Line::from(format!("SPD: {}", p.speed())),
        Line::from(format!("Pos: ({}, {})", p.x, p.y)),
        Line::from(format!("Room: {}", room_label)),
        Line::from(""),
    ];

    if world.inventory_open {
        text.push(Line::from(Span::styled(
            "Inventory",
            Style::default().fg(Color::Cyan),
        )));

        text.push(Line::from(vec![
            tab_label(InvTab::Weapons, inv.tab, "Weapons"),
            Span::raw(" "),
            tab_label(InvTab::Consumables, inv.tab, "Consumables"),
            Span::raw(" "),
            tab_label(InvTab::Backpack, inv.tab, "Backpack"),
        ]));
        text.push(Line::from(""));

        text.push(Line::from(Span::styled(
            "Weapons",
            Style::default().fg(Color::White),
        )));

        let sword_marker = if inv.tab == InvTab::Weapons
            && matches!(inv.selection(), InvSelection::SwordSlot)
        {
            ">"
        } else {
            " "
        };

        let sword_line = match &inv.sword {
            Some(sw) => {
                if inv.tab == InvTab::Weapons
                    && matches!(inv.selection(), InvSelection::SwordSlot)
                {
                    format!(
                        "{} Sword : {} ({} ATK, {} DEF, {} SPD, {} HP) [Space to unequip]",
                        sword_marker,
                        sw.name,
                        fmt_bonus(sw.atk_bonus),
                        fmt_bonus(sw.def_bonus),
                        fmt_bonus(sw.speed_bonus),
                        fmt_bonus(sw.hp_bonus),
                    )
                } else {
                    format!("{} Sword : {}", sword_marker, sw.name)
                }
            }
            None => format!("{} Sword : <empty>", sword_marker),
        };
        text.push(Line::from(sword_line));

        let shield_marker = if inv.tab == InvTab::Weapons
            && matches!(inv.selection(), InvSelection::ShieldSlot)
        {
            ">"
        } else {
            " "
        };

        let shield_line = match &inv.shield {
            Some(sh) => {
                if inv.tab == InvTab::Weapons
                    && matches!(inv.selection(), InvSelection::ShieldSlot)
                {
                    format!(
                        "{} Shield: {} ({} ATK, {} DEF, {} SPD, {} HP) [Space to unequip]",
                        shield_marker,
                        sh.name,
                        fmt_bonus(sh.atk_bonus),
                        fmt_bonus(sh.def_bonus),
                        fmt_bonus(sh.speed_bonus),
                        fmt_bonus(sh.hp_bonus),
                    )
                } else {
                    format!("{} Shield: {}", shield_marker, sh.name)
                }
            }
            None => format!("{} Shield: <empty>", shield_marker),
        };
        text.push(Line::from(shield_line));

        text.push(Line::from(""));

        text.push(Line::from(Span::styled(
            "Consumables (Space to use)",
            Style::default().fg(Color::White),
        )));

        if inv.consumables.is_empty() {
            let marker = if inv.tab == InvTab::Consumables { ">" } else { " " };
            text.push(Line::from(format!("{} <none>", marker)));
        } else {
            for (i, c) in inv.consumables.iter().enumerate() {
                let selected = inv.tab == InvTab::Consumables
                    && matches!(inv.selection(), InvSelection::Consumable(idx) if idx == i);

                let marker = if selected { ">" } else { " " };

                if selected {
                    text.push(Line::from(format!(
                        "{} {} ({} HP, {} ATK, {} DEF) [Space to use]",
                        marker,
                        c.name,
                        fmt_bonus(c.heal),
                        fmt_bonus(c.atk_bonus),
                        fmt_bonus(c.def_bonus),
                    )));
                } else {
                    text.push(Line::from(format!("{} {}", marker, c.name)));
                }
            }
        }

        let empty_slots = 10usize.saturating_sub(inv.consumables.len());
        text.push(Line::from(format!("Empty slots: {}", empty_slots)));

        text.push(Line::from(""));

        text.push(Line::from(Span::styled(
            "Backpack (Space to equip)",
            Style::default().fg(Color::White),
        )));

        if inv.backpack.is_empty() {
            let marker = if inv.tab == InvTab::Backpack { ">" } else { " " };
            text.push(Line::from(format!("{} <empty>", marker)));
        } else {
            for (i, b) in inv.backpack.iter().enumerate() {
                let marker = if inv.tab == InvTab::Backpack
                    && matches!(inv.selection(), InvSelection::BackpackItem(idx) if idx == i)
                {
                    ">"
                } else {
                    " "
                };
                text.push(Line::from(format!("{} {}", marker, b.name)));
            }
        }

        text.push(Line::from(""));
        text.push(Line::from("Up/Down: select"));
        text.push(Line::from("T: change tab"));
        text.push(Line::from("Space: use/unequip/equip"));
        text.push(Line::from("I or Esc: close"));
        text.push(Line::from("Q: stats"));
    } else {
        text.push(Line::from(Span::styled(
            "Controls",
            Style::default().fg(Color::Cyan),
        )));
        text.push(Line::from("WASD / Arrows: Move"));
        text.push(Line::from("E: Talk / Open chest"));
        text.push(Line::from("I: Inventory"));
        text.push(Line::from("T: Inventory Tab"));
        text.push(Line::from("Q: Stats"));
        text.push(Line::from("Ctrl+C: Quit"));
        text.push(Line::from("E on +: Switch rooms"));
    }

    let sidebar = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Player"))
        .wrap(Wrap { trim: true });

    f.render_widget(sidebar, area);
}

fn draw_logs(f: &mut Frame, area: Rect, world: &World) {
    f.render_widget(Clear, area);

    let mut lines: Vec<Line> = Vec::new();
    for msg in world.logs.iter() {
        lines.push(Line::from(msg.clone()));
    }

    let logs = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Log"))
        .wrap(Wrap { trim: true });

    f.render_widget(logs, area);
}

fn draw_stats(f: &mut Frame, area: Rect, world: &World) {
    let p = &world.player;
    let inv = &p.inventory;

    let sword = inv.sword.as_ref().map(|s| s.name.as_str()).unwrap_or("<empty>");
    let shield = inv.shield.as_ref().map(|s| s.name.as_str()).unwrap_or("<empty>");

    let lines = vec![
        Line::from(Span::styled(
            "Current Stats",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(format!("HP  : {}/{}", p.hp, p.max_hp)),
        Line::from(format!("ATK : {}", p.attack())),
        Line::from(format!("DEF : {}", p.defense())),
        Line::from(format!("SPD : {}", p.speed())),
        Line::from(""),
        Line::from(format!("Sword : {}", sword)),
        Line::from(format!("Shield: {}", shield)),
        Line::from(""),
        Line::from(Span::styled(
            "Press Q or Esc to close.",
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )),
    ];

    let stats = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Stats"))
        .wrap(Wrap { trim: true });

    f.render_widget(stats, area);
}

fn draw_dialogue(f: &mut Frame, area: Rect, world: &World) {
    let d = world.dialogue.as_ref().unwrap();
    let page_text = &d.pages[d.page_index];

    let mut lines: Vec<Line> = Vec::new();
    for raw_line in page_text.lines() {
        lines.push(Line::from(raw_line.to_string()));
    }

    let footer = if d.page_index + 1 < d.pages.len() {
        "Press SPACE to continue..."
    } else if d.awaiting.is_some() {
        "Enter your choice (letter)..."
    } else {
        "Press SPACE to close."
    };

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        footer,
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
    )));

    let dialog = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(d.title.clone()))
        .wrap(Wrap { trim: true });

    f.render_widget(dialog, area);
}