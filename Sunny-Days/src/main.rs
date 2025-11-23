mod engine;
mod map;
mod tui;
mod audio;

use engine::game_loop::run;

fn main() -> std::io::Result<()> {
    run()
}
