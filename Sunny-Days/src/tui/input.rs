use crossterm::event::{KeyEvent, KeyEventKind};

/// Returns true only for actual key presses (ignores repeats/releases).
pub fn is_press(key: &KeyEvent) -> bool {
    key.kind == KeyEventKind::Press
}
