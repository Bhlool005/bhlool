use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::process::Command;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const RESET: &str = "\x1b[0m";
const CLEAR: &str = "\x1b[2J\x1b[H";
const FG_WHITE: &str = "\x1b[97m";
const FG_GRAY: &str = "\x1b[90m";
const FG_RED: &str = "\x1b[91m";
const FG_GREEN: &str = "\x1b[92m";
const FG_YELLOW: &str = "\x1b[93m";
const FG_BLUE: &str = "\x1b[94m";
const FG_CYAN: &str = "\x1b[96m";
const BG_GRASS_A: &str = "\x1b[48;5;34m";
const BG_GRASS_B: &str = "\x1b[48;5;28m";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    food: Point,
    rocks: Vec<Point>,
    score: u32,
    seed: u64,
    over: bool,
    won: bool,
    paused: bool,
    mode: GameMode,
    speed: Speed,
}

impl Game {
    pub fn new(width: i32, height: i32) -> Self {
        let mut snake = VecDeque::new();
        let center = Point {
            x: width / 2,
            y: height / 2,
        };

        snake.push_front(center);
        snake.push_back(Point { x: center.x - 1, y: center.y });
        snake.push_back(Point { x: center.x - 2, y: center.y });

        let mut game = Self {
            width,
            height,
            snake,
            direction: Direction::Right,
            food: Point { x: 0, y: 0 },
            rocks: Vec::new(),
            score: 0,
            seed: 0x420_2026,
            over: false,
            won: false,
            paused: false,
            mode: GameMode::WrapAround,
            speed: Speed::Normal,
        };

        game.rocks = game.spawn_rocks(8);
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

    fn spawn_rocks(&mut self, count: usize) -> Vec<Point> {
        let mut rocks = Vec::new();

        while rocks.len() < count {
            let p = self.random_point();

            if self.snake.contains(&p) || rocks.contains(&p) {
                continue;
            }

            rocks.push(p);
        }

        rocks
    }

    fn spawn_food(&mut self) -> Point {
        loop {
            let p = self.random_point();

            if !self.snake.contains(&p) && !self.rocks.contains(&p) {
                return p;
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

        if next.x >= 0 && next.y >= 0 && next.x < self.width && next.y < self.height {
            Some(next)
        } else {
            None
        }
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

        if self.snake.contains(&next) {
            self.over = true;
            return;
        }

        self.snake.push_front(next);

        if next == self.food {
            self.score += self.speed.bonus();
            self.food = self.spawn_food();
        } else {
            self.snake.pop_back();
        }
    }

    fn render(&self) -> String {
        let mut out = String::new();

        out.push_str(CLEAR);

        out.push_str(FG_GRAY);
        out.push('+');

        for _ in 0..self.width {
            out.push_str("--");
        }

        out.push_str("+\n");

        for y in 0..self.height {
            out.push_str("|");

            for x in 0..self.width {
                let p = Point { x, y };

                if p == self.head() {
                    out.push_str(FG_BLUE);
                    out.push_str("██");
                } else if p == self.food {
                    out.push_str(FG_RED);
                    out.push_str("● ");
                } else if self.snake.iter().skip(1).any(|s| *s == p) {
                    out.push_str(FG_CYAN);
                    out.push_str("▓▓");
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
            out.push_str("|\n");
        }

        out.push_str(FG_GRAY);
        out.push('+');

        for _ in 0..self.width {
            out.push_str("--");
        }

        out.push_str("+\n");

        out.push_str(FG_YELLOW);
        out.push_str(&format!(
            "Score: {}   Speed: {}   Mode: {}\n",
            self.score,
            self.speed.label(),
            self.mode.as_str()
        ));

        out.push_str(FG_GREEN);
        out.push_str("Controls: W/A/S/D move, M mode, T speed, P pause, Q quit\n");

        out.push_str(RESET);

        out
    }
}

fn parse_input_byte(byte: u8) -> Option<char> {
    match byte {
        b'w' | b'W' => Some('w'),
        b'a' | b'A' => Some('a'),
        b's' | b'S' => Some('s'),
        b'd' | b'D' => Some('d'),
        b'm' | b'M' => Some('m'),
        b't' | b'T' => Some('t'),
        b'p' | b'P' => Some('p'),
        b'q' | b'Q' => Some('q'),
        _ => None,
    }
}

fn try_enable_raw_stdin() -> Option<String> {
    let saved = Command::new("stty").arg("-g").output().ok()?;

    Command::new("stty")
        .args(["raw", "-echo", "min", "0", "time", "0"])
        .status()
        .ok()?;

    String::from_utf8(saved.stdout).ok().map(|s| s.trim().to_owned())
}

fn try_restore_stdin(config: &str) {
    let _ = Command::new("stty").arg(config).status();
}

struct RawModeGuard {
    state: Option<String>,
}

impl RawModeGuard {
    fn new() -> Self {
        Self {
            state: try_enable_raw_stdin(),
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        if let Some(state) = &self.state {
            try_restore_stdin(state);
        }
    }
}

impl Game {
    fn handle_command(&mut self, cmd: char) -> bool {
        match cmd {
            'w' => self.set_direction(Direction::Up),
            'a' => self.set_direction(Direction::Left),
            's' => self.set_direction(Direction::Down),
            'd' => self.set_direction(Direction::Right),
            'm' => self.mode = self.mode.toggle(),
            't' => self.speed = self.speed.toggle(),
            'p' => self.paused = !self.paused,
            'q' => return true,
            _ => {}
        }
        false
    }
}

pub fn run() -> io::Result<()> {
    let mut game = Game::new(20, 12);
    let _raw_mode_guard = RawModeGuard::new();

    let (tx, rx) = mpsc::channel::<u8>();

    thread::spawn(move || {
        let stdin = io::stdin();
        let mut lock = stdin.lock();
        let mut buf = [0_u8; 1];

        loop {
            match lock.read(&mut buf) {
                Ok(1) => {
                    if tx.send(buf[0]).is_err() {
                        break;
                    }
                }
                _ => {}
            }
        }
    });

    loop {
        print!("{}", game.render());
        io::stdout().flush()?;

        let tick_delay = game.speed.tick_delay();

        if let Ok(input) = rx.recv_timeout(tick_delay) {
            if let Some(cmd) = parse_input_byte(input) {
                if game.handle_command(cmd) {
                    break;
                }
            }
        }

        if !game.over {
            game.step();
        }
    }

    println!("Final Score: {}", game.score);
    Ok(())
}
