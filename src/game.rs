use std::collections::{HashSet, VecDeque};
use std::io::{self, Write};
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
            GameMode::WrapAround => "Wrap-around",
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

#[derive(Clone, Copy)]
enum Tile {
    Empty,
    Food,
    Rock,
    SnakeBody,
    SnakeHead,
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
    won: bool,
    paused: bool,
    mode: GameMode,
    speed: Speed,
}

impl Game {
    pub fn new(width: i32, height: i32) -> Self {
        let width = width.max(MIN_BOARD_SIZE);
        let height = height.max(MIN_BOARD_SIZE);

        let center = Point {
            x: width / 2,
            y: height / 2,
        };

        let mut snake = VecDeque::new();
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
            won: false,
            paused: false,
            mode: GameMode::WrapAround,
            speed: Speed::Normal,
        };

        game.rocks = game.spawn_rocks(DEFAULT_ROCKS);
        game.food = game.spawn_food();

        game
    }

    fn board_capacity(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn head(&self) -> Point {
        *self.snake.front().expect("snake has at least one segment")
    }

    fn next_rand(&mut self) -> u64 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
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
            if !self.snake.contains(&p) && !rocks.contains(&p) {
                rocks.push(p);
            }
        }

        rocks
    }

    fn spawn_food(&mut self) -> Option<Point> {
        if self.snake.len() + self.rocks.len() >= self.board_capacity() {
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

        let grows = self.food == Some(next);

        if self.snake.contains(&next) {
            self.over = true;
            return;
        }

        self.snake.push_front(next);

        if grows {
            self.score += self.speed.bonus();
            self.food = self.spawn_food();
            if self.food.is_none() {
                self.over = true;
                self.won = true;
            }
        } else {
            self.snake.pop_back();
        }
    }

    fn point_to_index(&self, p: Point) -> usize {
        (p.y * self.width + p.x) as usize
    }

    fn build_tile_map(&self) -> Vec<Tile> {
        let mut tiles = vec![Tile::Empty; self.board_capacity()];

        for rock in &self.rocks {
            let index = self.point_to_index(*rock);
            tiles[index] = Tile::Rock;
        }

        for segment in self.snake.iter().skip(1) {
            let index = self.point_to_index(*segment);
            tiles[index] = Tile::SnakeBody;
        }

        if let Some(food) = self.food {
            let index = self.point_to_index(food);
            tiles[index] = Tile::Food;
        }

        let head_index = self.point_to_index(self.head());
        tiles[head_index] = Tile::SnakeHead;

        tiles
    }

    fn render(&self) -> String {
        let mut out = String::new();

        out.push_str(FG_GRAY);
        out.push('+');
        for _ in 0..self.width {
            out.push_str("--");
        }
        out.push_str("+\n");

        let head = self.head();
        let body: HashSet<Point> = self.snake.iter().skip(1).copied().collect();
        let rocks: HashSet<Point> = self.rocks.iter().copied().collect();
        let tiles = self.build_tile_map();

        for y in 0..self.height {
            out.push_str(FG_GRAY);
            out.push('|');

            for x in 0..self.width {
                let p = Point { x, y };
                let tile = tiles[self.point_to_index(p)];

                if p == head {
                    out.push_str(FG_BLUE);
                    out.push_str("██");
                    continue;
                }

                if self.food == Some(p) {
                    out.push_str(FG_RED);
                    out.push_str("● ");
                    continue;
                }

                if body.contains(&p) || matches!(tile, Tile::SnakeBody) {
                    out.push_str(FG_CYAN);
                    out.push_str("▓▓");
                    continue;
                }

                if rocks.contains(&p) || matches!(tile, Tile::Rock) {
                    out.push_str(FG_WHITE);
                    out.push_str("◼ ");
                    continue;
                }

                if (x + y) % 2 == 0 {
                    out.push_str(BG_GRASS_A);
                } else {
                    out.push_str(BG_GRASS_B);
                }

                out.push_str("  ");
            }

            out.push_str(RESET);
            out.push_str(FG_GRAY);
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
        out.push_str("Controls: W/A/S/D = move | M = mode | T = speed | P = pause | Q = quit\n");

        if self.paused {
            out.push_str(FG_YELLOW);
            out.push_str("Paused. Press P to continue.\n");
        }

        if self.over {
            if self.won {
                out.push_str(FG_GREEN);
                out.push_str("You filled the map. You win!\n");
            } else {
                out.push_str(FG_RED);
                out.push_str("Game Over!\n");
            }
            out.push_str(FG_WHITE);
            out.push_str("Press Q to exit.\n");
        }

        out.push_str(RESET);

        out
    }

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
        print!("{}", CLEAR);
        print!("{}", game.render());
        io::stdout().flush()?;

        let tick = game.speed.tick_delay();

        match rx.recv_timeout(tick) {
            Ok(input) => {
                if let Some(cmd) = parse_command(&input) {
                    if game.handle_command(cmd) {
                        break;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        game.step();
    }

    print!("{}", CLEAR);
    println!(
        "{}Thanks for playing! Final score: {}{}",
        FG_CYAN, game.score, RESET
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_command_ignores_whitespace() {
        assert_eq!(parse_command("  D\n"), Some('d'));
    }

    #[test]
    fn game_clamps_small_board_to_min() {
        let game = Game::new(1, 1);
        assert_eq!(game.width, MIN_BOARD_SIZE);
        assert_eq!(game.height, MIN_BOARD_SIZE);
        assert_eq!(game.snake.len(), 3);
    }
}
