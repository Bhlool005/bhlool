mod game;

use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};
use std::{thread, time::Duration};
use game::{Game, Direction};

fn main() {
    enable_raw_mode().unwrap();

    loop {
        let mut game = Game::new();

        loop {
            if event::poll(Duration::from_millis(0)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('w') => game.change_direction(Direction::Up),
                        KeyCode::Down | KeyCode::Char('s') => game.change_direction(Direction::Down),
                        KeyCode::Left | KeyCode::Char('a') => game.change_direction(Direction::Left),
                        KeyCode::Right | KeyCode::Char('d') => game.change_direction(Direction::Right),
                        KeyCode::Esc => { disable_raw_mode().unwrap(); return; },
                        _ => {}
                    }
                }
            }

            game.update();
            game.update_ai();
            game.check_collisions();
            game.render();

            if game.game_over {
                println!("Game Over! Score: {}", game.score);
                println!("Press R to Play Again or ESC to Quit");

                loop {
                    if let Event::Key(key) = event::read().unwrap() {
                        match key.code {
                            KeyCode::Char('r') => break,
                            KeyCode::Esc => { disable_raw_mode().unwrap(); return; },
                            _ => {}
                        }
                    }
                }
                break;
            }

            thread::sleep(Duration::from_millis(game.speed));
        }
    }
}