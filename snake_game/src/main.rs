use std::io::{stdout, Write};
use std::time::{Duration, Instant};
use std::thread::sleep;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute,
    terminal::{self, ClearType},
    style::Print,
};

use rand::Rng;

const WIDTH: u16 = 40;
const HEIGHT: u16 = 20;

#[derive(Clone, Copy)]
struct Point {
    x: u16,
    y: u16,
}

fn draw_border() {
    let mut stdout = stdout();

    for x in 0..=WIDTH {
        execute!(stdout, cursor::MoveTo(x, 0), Print("#")).unwrap();
        execute!(stdout, cursor::MoveTo(x, HEIGHT), Print("#")).unwrap();
    }

    for y in 0..=HEIGHT {
        execute!(stdout, cursor::MoveTo(0, y), Print("#")).unwrap();
        execute!(stdout, cursor::MoveTo(WIDTH, y), Print("#")).unwrap();
    }
}

fn draw_point(p: Point, ch: char) {
    let mut stdout = stdout();
    execute!(stdout, cursor::MoveTo(p.x, p.y), Print(ch)).unwrap();
}

fn random_food() -> Point {
    let mut rng = rand::thread_rng();
    Point {
        x: rng.gen_range(1..WIDTH),
        y: rng.gen_range(1..HEIGHT),
    }
}

fn run_game() -> bool {
    let mut stdout = stdout();

    let mut snake = vec![Point { x: 5, y: 5 }];
    let mut food = random_food();
    let mut dir = (1i16, 0i16);
    let mut last_tick = Instant::now();

    loop {
        execute!(stdout, terminal::Clear(ClearType::All)).unwrap();
        draw_border();

        draw_point(food, '*');
        for s in &snake {
            draw_point(*s, 'O');
        }

        if event::poll(Duration::from_millis(0)).unwrap() {
            if let Event::Key(k) = event::read().unwrap() {
                dir = match k.code {
                    KeyCode::Up => (0, -1),
                    KeyCode::Down => (0, 1),
                    KeyCode::Left => (-1, 0),
                    KeyCode::Right => (1, 0),
                    KeyCode::Esc => return false,
                    _ => dir,
                };
            }
        }

        if last_tick.elapsed() >= Duration::from_millis(150) {
            let mut head = snake[0];
            head.x = (head.x as i16 + dir.0) as u16;
            head.y = (head.y as i16 + dir.1) as u16;

            if head.x == 0 || head.x == WIDTH || head.y == 0 || head.y == HEIGHT {
                return true; // GAME OVER
            }

            snake.insert(0, head);

            if head.x == food.x && head.y == food.y {
                food = random_food();
            } else {
                snake.pop();
            }

            last_tick = Instant::now();
        }

        stdout.flush().unwrap();
        sleep(Duration::from_millis(10));
    }
}

fn game_over_screen() -> bool {
    let mut stdout = stdout();

    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(10, 8),
        Print("GAME OVER"),
        cursor::MoveTo(6, 10),
        Print("Press R to restart"),
        cursor::MoveTo(6, 11),
        Print("Press ESC to quit")
    )
    .unwrap();

    loop {
        if let Event::Key(k) = event::read().unwrap() {
            match k.code {
                KeyCode::Char('r') | KeyCode::Char('R') => return true,
                KeyCode::Esc => return false,
                _ => {}
            }
        }
    }
}

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    loop {
        let died = run_game();
        if died {
            if !game_over_screen() {
                break;
            }
        } else {
            break;
        }
    }

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
