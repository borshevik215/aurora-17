mod audio;
mod commands;
mod engine;
mod game;
mod ship;
mod systems;
mod terminal;

use macroquad::prelude::*;

fn window_conf() -> Conf {
    Conf {
        window_title:
            "AURORA-17 // REMOTE OPERATIONS TERMINAL"
                .to_owned(),

        window_width: 1280,
        window_height: 720,

        window_resizable: true,
        high_dpi: false,

        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let seed = std::env::args().nth(1).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0x7F2A91C4);

    let mut game =
        game::Game::new(seed).await;

    loop {
        game.update();
        game.draw();

        next_frame().await;
    }
}
