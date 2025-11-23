use crate::engine::action::Action;
use crate::engine::world::World;
use crate::tui::{input::is_press, renderer::render};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ratatui::{backend::CrosstermBackend, Terminal};

use std::{
    io,
    time::{Duration, Instant},
};

const MOVE_COOLDOWN_MS: u64 = 90;

pub fn run() -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let seed = rand::random::<u64>();
    let mut world = World::new(seed, 80, 45);

    let tick_rate = Duration::from_millis(60);
    let mut last_move_time = Instant::now() - Duration::from_millis(MOVE_COOLDOWN_MS);

    let mut running = true;
    while running {
        if let Err(_) = terminal.draw(|f| render(f, &world)) {
            terminal.autoresize()?;
            terminal.clear()?;
            continue;
        }

        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Resize(_, _) => {
                    terminal.autoresize()?;
                    terminal.clear()?;
                }

                Event::Key(key) => {
                    if !is_press(&key) {
                        continue;
                    }

                    let mut action = if world.inventory_open {
                        match key.code {
                            KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Esc => Action::ToggleInventory,
                            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Action::InventoryUp,
                            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Action::InventoryDown,
                            KeyCode::Char(' ') => Action::UseConsumable,
                            KeyCode::Char('q') | KeyCode::Char('Q') => Action::Quit,
                            _ => Action::None,
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') => Action::Quit,
                            KeyCode::Char('i') | KeyCode::Char('I') => Action::ToggleInventory,

                            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Action::Move(0, -1),
                            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Action::Move(0, 1),
                            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => Action::Move(-1, 0),
                            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => Action::Move(1, 0),

                            _ => Action::None,
                        }
                    };

                    if let Action::Move(_, _) = action {
                        let now = Instant::now();
                        if now.duration_since(last_move_time) < Duration::from_millis(MOVE_COOLDOWN_MS) {
                            action = Action::None;
                        } else {
                            last_move_time = now;
                        }
                    }

                    running = world.apply_action(action);
                }

                _ => {}
            }
        } else {
            running = world.apply_action(Action::None);
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
