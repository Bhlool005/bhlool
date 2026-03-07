mod game;
mod leaderboard;

use game::Game;
use std::{thread, time};

fn main() {

    let mut game = Game::new();

    loop {

        game.update();
        game.render();

        if game.game_over {
            println!("Game Over!");
            break;
        }

        thread::sleep(time::Duration::from_millis(200));
    }

}
