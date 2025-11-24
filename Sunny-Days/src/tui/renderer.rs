use crate::engine::world::{World, GameState};
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
    px: i32,
    py: i32,
    map_w: i32,
    map_h: i32,
    view_w: i32,
    view_h: i32,
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
        GameState::Playing => draw_playing(f, size, world),
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

    draw_logs(f, bottom, world);
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

            if wx < 0 || wy < 0 || wx >= map_w || wy >= map_h {
                spans.push(Span::raw(" "));
                continue;
            }

            let tile = map.get(wx as usize, wy as usize);
            let (ch, style) = match tile {
                Tile::Wall => ("#", Style::default().fg(Color::DarkGray)),
                Tile::Floor => (" ", Style::default()),
                Tile::Door => ("+", Style::default().fg(Color::White)),
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

fn draw_sidebar(f: &mut Frame, area: Rect, world: &World) {
    f.render_widget(Clear, area);

    let p = &world.player;
    let room_label = if world.current == 0 { "Room 1" } else { "Room 2" };

    let mut text: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("HP: ", Style::default().fg(Color::White)),
            Span::styled(format!("{}/{}", p.hp, p.max_hp), Style::default().fg(Color::Green)),
        ]),
        Line::from(format!("Pos: ({}, {})", p.x, p.y)),
        Line::from(format!("Room: {}", room_label)),
        Line::from(format!("Seed: {}", world.seed)),
        Line::from(""),
    ];

    if world.inventory_open {
        text.push(Line::from(Span::styled("Inventory", Style::default().fg(Color::Cyan))));
        let sword_name = p.inventory.sword.as_ref().map(|e| e.name.as_str()).unwrap_or("<empty>");
        let shield_name = p.inventory.shield.as_ref().map(|e| e.name.as_str()).unwrap_or("<empty>");
        text.push(Line::from(format!("Sword : {}", sword_name)));
        text.push(Line::from(format!("Shield: {}", shield_name)));
        text.push(Line::from(""));

        text.push(Line::from("Consumables (Space to use)"));
        if p.inventory.consumables.is_empty() {
            text.push(Line::from("  <none>"));
        } else {
            for (i, c) in p.inventory.consumables.iter().enumerate() {
                let marker = if i == p.inventory.selected { ">" } else { " " };
                text.push(Line::from(format!("{} {}", marker, c.name)));
            }
        }

        let empty_slots = 10usize.saturating_sub(p.inventory.consumables.len());
        text.push(Line::from(format!("Empty slots: {}", empty_slots)));
        text.push(Line::from(""));
        text.push(Line::from("Up/Down: select"));
        text.push(Line::from("Space: use"));
        text.push(Line::from("I or Esc: close"));
    } else {
        text.push(Line::from(Span::styled("Controls", Style::default().fg(Color::Cyan))));
        text.push(Line::from("WASD / Arrows: Move"));
        text.push(Line::from("I: Inventory"));
        text.push(Line::from("Q: Quit"));
        text.push(Line::from("Step on + to switch rooms"));
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
