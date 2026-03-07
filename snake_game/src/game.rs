use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";
const FG_WHITE: &str = "\x1b[97m";
const FG_RED: &str = "\x1b[91m";
const FG_YELLOW: &str = "\x1b[93m";
const FG_BLUE: &str = "\x1b[94m";
const FG_CYAN: &str = "\x1b[96m";
const BG_GRASS_A: &str = "\x1b[48;5;34m";
const BG_GRASS_B: &str = "\x1b[48;5;28m";

const DEFAULT_WIDTH: i32 = 20;
const DEFAULT_HEIGHT: i32 = 12;
const DEFAULT_ROCKS: usize = 8;
const MIN_BOARD_SIZE: i32 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn is_opposite(self, other: Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GameMode {
    Classic,
    WrapAround,
}

impl GameMode {
    fn toggle(self) -> Self {
        match self {
            GameMode::Classic => GameMode::WrapAround,
            GameMode::WrapAround => GameMode::Classic,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            GameMode::Classic => "Classic",
            GameMode::WrapAround => "WrapAround",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Speed {
    Chill,
    Normal,
    Turbo,
}

impl Speed {
    fn toggle(self) -> Self {
        match self {
            Speed::Chill => Speed::Normal,
            Speed::Normal => Speed::Turbo,
            Speed::Turbo => Speed::Chill,
        }
    }

    fn bonus(self) -> u32 {
        match self {
            Speed::Chill => 8,
            Speed::Normal => 12,
            Speed::Turbo => 18,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Speed::Chill => "Chill",
            Speed::Normal => "Normal",
            Speed::Turbo => "Turbo",
        }
    }

    fn tick_delay(self) -> Duration {
        match self {
            Speed::Chill => Duration::from_millis(220),
            Speed::Normal => Duration::from_millis(150),
            Speed::Turbo => Duration::from_millis(90),
        }
    }
}

pub struct Game {
    width: i32,
    height: i32,
    snake: VecDeque<Point>,
    direction: Direction,
    food: Option<Point>,
    rocks: Vec<Point>,
    score: u32,
    seed: u64,
    over: bool,
    paused: bool,
    mode: GameMode,
    speed: Speed,
}

impl Game {
    pub fn new(width: i32, height: i32) -> Self {
        let width = width.max(MIN_BOARD_SIZE);
        let height = height.max(MIN_BOARD_SIZE);

        let mut snake = VecDeque::new();
        let center = Point {
            x: width / 2,
            y: height / 2,
        };

        snake.push_front(center);
        snake.push_back(Point {
            x: center.x - 1,
            y: center.y,
        });
        snake.push_back(Point {
            x: center.x - 2,
            y: center.y,
        });

        let mut game = Self {
            width,
            height,
            snake,
            direction: Direction::Right,
            food: None,
            rocks: Vec::new(),
            score: 0,
            seed: 0x420_2026,
            over: false,
            paused: false,
            mode: GameMode::WrapAround,
            speed: Speed::Normal,
        };

        game.rocks = game.spawn_rocks(DEFAULT_ROCKS);
        game.food = game.spawn_food();

        game
    }

    fn head(&self) -> Point {
        *self.snake.front().unwrap()
    }

    fn next_rand(&mut self) -> u64 {
        self.seed = self.seed.wrapping_mul(1664525).wrapping_add(1013904223);
        self.seed
    }

    fn random_point(&mut self) -> Point {
        Point {
            x: (self.next_rand() % self.width as u64) as i32,
            y: (self.next_rand() % self.height as u64) as i32,
        }
    }

    fn board_capacity(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn occupied_count(&self) -> usize {
        self.snake.len() + self.rocks.len()
    }

    fn spawn_rocks(&mut self, count: usize) -> Vec<Point> {
        let max_rocks = self.board_capacity().saturating_sub(self.snake.len());
        let target = count.min(max_rocks);
        let mut rocks = Vec::new();

        while rocks.len() < target {
            let p = self.random_point();

            if self.snake.contains(&p) || rocks.contains(&p) {
                continue;
            }

            rocks.push(p);
        }

        rocks
    }

    fn spawn_food(&mut self) -> Option<Point> {
        if self.occupied_count() >= self.board_capacity() {
            return None;
        }

        loop {
            let p = self.random_point();

            if !self.snake.contains(&p) && !self.rocks.contains(&p) {
                return Some(p);
            }
        }
    }

    fn set_direction(&mut self, next: Direction) {
        if !self.direction.is_opposite(next) {
            self.direction = next;
        }
    }

    fn next_head(&self) -> Option<Point> {
        let mut next = self.head();

        match self.direction {
            Direction::Up => next.y -= 1,
            Direction::Down => next.y += 1,
            Direction::Left => next.x -= 1,
            Direction::Right => next.x += 1,
        }

        if self.mode == GameMode::WrapAround {
            if next.x < 0 {
                next.x = self.width - 1;
            }
            if next.y < 0 {
                next.y = self.height - 1;
            }
            if next.x >= self.width {
                next.x = 0;
            }
            if next.y >= self.height {
                next.y = 0;
            }

            return Some(next);
        }

        if next.x < 0 || next.y < 0 || next.x >= self.width || next.y >= self.height {
            return None;
        }

        Some(next)
    }

    fn step(&mut self) {
        if self.over || self.paused {
            return;
        }

        let Some(next) = self.next_head() else {
            self.over = true;
            return;
        };

        if self.rocks.contains(&next) {
            self.over = true;
            return;
        }

        let will_grow = self.food == Some(next);
        let tail = *self.snake.back().expect("snake has at least one segment");
        let hits_body = if will_grow {
            self.snake.contains(&next)
        } else {
            self.snake
                .iter()
                .any(|segment| *segment != tail && *segment == next)
        };

        if hits_body {
            self.over = true;
            return;
        }

        self.snake.push_front(next);

        if will_grow {
            self.score += self.speed.bonus();
            self.food = self.spawn_food();
        } else {
            self.snake.pop_back();
        }
    }

    fn render(&self) -> String {
        let mut out = String::new();

        out.push_str(CLEAR);

        out.push_str(&format!(
            "{}Score: {}  Speed: {}  Mode: {}\n",
            FG_YELLOW,
            self.score,
            self.speed.label(),
            self.mode.as_str()
        ));

        out.push_str(
            "Controls: W A S D move | M mode | T speed | P pause | R restart | Q quit\n\n",
        );

        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };

                if p == self.head() {
                    out.push_str(FG_BLUE);
                    out.push_str("██");
                } else if self.snake.contains(&p) {
                    out.push_str(FG_CYAN);
                    out.push_str("▓▓");
                } else if self.food == Some(p) {
                    out.push_str(FG_RED);
                    out.push_str("● ");
                } else if self.rocks.contains(&p) {
                    out.push_str(FG_WHITE);
                    out.push_str("◼ ");
                } else {
                    if (x + y) % 2 == 0 {
                        out.push_str(BG_GRASS_A);
                    } else {
                        out.push_str(BG_GRASS_B);
                    }
                    out.push_str("  ");
                }
            }
            out.push_str(RESET);
            out.push('\n');
        }

        if self.over {
            out.push_str(FG_RED);
            out.push_str("\nGame Over!\n");
            out.push_str(FG_WHITE);
            out.push_str("Press R to play again or Q to quit\n");
        }

        out.push_str(RESET);

        out
    }

    fn handle_command(&mut self, cmd: char) -> Option<&'static str> {
        match cmd {
            'w' => self.set_direction(Direction::Up),
            'a' => self.set_direction(Direction::Left),
            's' => self.set_direction(Direction::Down),
            'd' => self.set_direction(Direction::Right),
            'm' => self.mode = self.mode.toggle(),
            't' => self.speed = self.speed.toggle(),
            'p' => self.paused = !self.paused,
            'r' => return Some("restart"),
            'q' => return Some("quit"),
            _ => {}
        }

        None
    }
}

fn parse_command(input: &str) -> Option<char> {
    input
        .chars()
        .find(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_lowercase())
}

pub fn run() -> io::Result<()> {
    let mut game = Game::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let stdin = io::stdin();
        loop {
            let mut input = String::new();
            if stdin.read_line(&mut input).is_err() {
                break;
            }
            if tx.send(input).is_err() {
                break;
            }
        }
    });

    loop {
        print!("{}", game.render());
        io::stdout().flush()?;

        let tick_delay = game.speed.tick_delay();

        match rx.recv_timeout(tick_delay) {
            Ok(input) => {
                if let Some(cmd) = parse_command(&input) {
                    if let Some(action) = game.handle_command(cmd) {
                        match action {
                            "quit" => break,
                            "restart" => game = Game::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
                            _ => {}
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        game.step();
    }

    println!("{}Thanks for playing!{}", FG_CYAN, RESET);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_rocks_caps_to_available_cells() {
        let mut game = Game::new(5, 5);
        let rocks = game.spawn_rocks(usize::MAX);
        assert_eq!(rocks.len(), game.board_capacity() - game.snake.len());
    }

    #[test]
    fn step_allows_moving_into_tail_when_not_growing() {
        let mut game = Game::new(5, 5);
        game.snake = VecDeque::from([
            Point { x: 2, y: 1 },
            Point { x: 2, y: 2 },
            Point { x: 1, y: 2 },
            Point { x: 1, y: 1 },
        ]);
        game.direction = Direction::Left;
        game.rocks.clear();
        game.food = Some(Point { x: 4, y: 4 });

        game.step();

        assert!(!game.over);
        assert_eq!(game.head(), Point { x: 1, y: 1 });
        assert_eq!(game.snake.len(), 4);
    }

    #[test]
    fn step_detects_body_collision_when_growing() {
        let mut game = Game::new(5, 5);
        game.snake = VecDeque::from([
            Point { x: 2, y: 1 },
            Point { x: 2, y: 2 },
            Point { x: 1, y: 2 },
            Point { x: 1, y: 1 },
        ]);
        game.direction = Direction::Left;
        game.rocks.clear();
        game.food = Some(Point { x: 1, y: 1 });

        game.step();

        assert!(game.over);
    }
}
