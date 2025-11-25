use crate::audio::Music;
use crate::engine::action::Action;
use crate::engine::world::{World, GameState};
use crate::tui::{input::is_press, renderer::render};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
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
    let _music = match Music::start_loop("assets/Background1.mp3") {
        Ok(m) => Some(m),
        Err(e) => {
            eprintln!("Audio disabled: {e}");
            None
        }
    };

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
    
    // Track last battle input for 10s penalty
    let mut last_battle_input = Instant::now();

    let mut running = true;
    while running {
        // Check Death
        if world.player.hp <= 0 {
            terminal.clear()?;
            println!("You died. Press Ctrl+C to quit.");
            // Break loop to exit properly
            break;
        }

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

                    // Quit with Ctrl+C anywhere
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        if let KeyCode::Char('c') | KeyCode::Char('q') = key.code {
                            running = world.apply_action(Action::Quit);
                            continue;
                        }
                    }

                    let mut action = match world.state {
                        GameState::Title | GameState::Intro => match key.code {
                            KeyCode::Char(' ') | KeyCode::Enter => Action::Confirm,
                            _ => Action::None,
                        },

                        GameState::Dialogue => match key.code {
                            KeyCode::Char(' ') | KeyCode::Enter => Action::Confirm,
                            KeyCode::Char(c) if c.is_ascii_alphabetic() => Action::Choice(c),
                            _ => Action::None,
                        },

                        GameState::Battle => {
                            if world.inventory_open {
                                match key.code {
                                    KeyCode::Char('i') | KeyCode::Esc => Action::ToggleInventory,
                                    KeyCode::Up => Action::InventoryUp,
                                    KeyCode::Down => Action::InventoryDown,
                                    KeyCode::Char(' ') => Action::UseConsumable,
                                    _ => Action::None,
                                }
                            } else {
                                let now = Instant::now();
                                let elapsed = now.duration_since(last_battle_input);
                                let penalty = elapsed.as_secs() >= 10;
                                
                                let act = match key.code {
                                    KeyCode::Char('1') => Action::BattleOption(1, penalty),
                                    KeyCode::Char('2') => Action::BattleOption(2, penalty),
                                    KeyCode::Char('3') => Action::BattleOption(3, penalty),
                                    _ => Action::None,
                                };
                                
                                if !matches!(act, Action::None) {
                                    last_battle_input = now;
                                }
                                act
                            }
                        },

                        GameState::Playing => {
                            if world.stats_open {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Action::ToggleStats,
                                    _ => Action::None,
                                }
                            } else if world.inventory_open {
                                match key.code {
                                    KeyCode::Char('t') | KeyCode::Char('T') => Action::ToggleInvTab, // NEW
                                    KeyCode::Char('q') | KeyCode::Char('Q') => Action::ToggleStats,
                                    KeyCode::Char('i') | KeyCode::Char('I') | KeyCode::Esc => Action::ToggleInventory,
                                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Action::InventoryUp,
                                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Action::InventoryDown,
                                    KeyCode::Char(' ') => Action::UseConsumable,
                                    _ => Action::None,
                                }
                            } else {
                                match key.code {
                                    KeyCode::Char('q') | KeyCode::Char('Q') => Action::ToggleStats,
                                    KeyCode::Char('i') | KeyCode::Char('I') => Action::ToggleInventory,
                                    KeyCode::Char('e') | KeyCode::Char('E') => Action::Interact,

                                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => Action::Move(0, -1),
                                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => Action::Move(0, 1),
                                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => Action::Move(-1, 0),
                                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => Action::Move(1, 0),

                                    _ => Action::None,
                                }
                            }
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

                    // If transitioning INTO Battle, reset timer
                    let old_state = world.state.clone();
                    running = world.apply_action(action);
                    if old_state != GameState::Battle && world.state == GameState::Battle {
                        last_battle_input = Instant::now();
                    }
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
    
    if world.player.hp <= 0 {
        println!("You died.");
    }

    Ok(())
}