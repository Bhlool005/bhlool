use std::collections::VecDeque;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
        let width = width.max(1);
        let height = height.max(1);

        let center = Point {
            x: width / 2,
            y: height / 2,
        };

        let mut snake = VecDeque::new();
        snake.push_back(center);
        if center.x > 0 {
            snake.push_back(Point {
                x: center.x - 1,
                y: center.y,
            });
        }
        if center.x > 1 {
            snake.push_back(Point {
                x: center.x - 2,
                y: center.y,
            });
        }

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
        if game.food.is_none() {
            game.over = true;
            game.won = true;
        }

        game
    }

    fn board_capacity(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn occupied_count(&self) -> usize {
        self.snake.len() + self.rocks.len()
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

    fn is_snake_at(&self, p: Point) -> bool {
        self.snake.iter().any(|seg| *seg == p)
    }

    fn is_rock_at(&self, p: Point) -> bool {
        self.rocks.iter().any(|rock| *rock == p)
    }

    fn spawn_rocks(&mut self, requested: usize) -> Vec<Point> {
        let mut rocks = Vec::new();
        if self.board_capacity() <= self.snake.len() {
            return rocks;
        }

        let max_placeable = self.board_capacity().saturating_sub(self.snake.len() + 1);
        let target = requested.min(max_placeable);
        let mut attempts = 0usize;
        let max_attempts = self.board_capacity().saturating_mul(24).max(1);

        while rocks.len() < target && attempts < max_attempts {
            attempts += 1;
            let p = self.random_point();
            let head = self.head();
            let too_close = (p.x - head.x).abs() <= 2 && (p.y - head.y).abs() <= 2;
            if self.is_snake_at(p) || rocks.contains(&p) || too_close {
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

        let max_attempts = self.board_capacity().saturating_mul(4).max(1);
        for _ in 0..max_attempts {
            let p = self.random_point();
            if !self.is_snake_at(p) && !self.is_rock_at(p) {
                return Some(p);
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };
                if !self.is_snake_at(p) && !self.is_rock_at(p) {
                    return Some(p);
                }
            }
        }

        None
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

        (next.x >= 0 && next.y >= 0 && next.x < self.width && next.y < self.height).then_some(next)
    }

    fn step(&mut self) {
        if self.over || self.paused {
            return;
        }

        let Some(next) = self.next_head() else {
            self.over = true;
            return;
        };

        if self.is_rock_at(next) {
            self.over = true;
            return;
        }

        let tail = *self.snake.back().expect("snake has tail");
        let grows = self.food == Some(next);
        if self.is_snake_at(next) && (grows || next != tail) {
            self.over = true;
            return;
        }

        self.snake.push_front(next);

        if grows {
            self.score += self.speed.bonus();

            if self.occupied_count() >= self.board_capacity() {
                self.food = None;
                self.over = true;
                self.won = true;
                return;
            }

            self.food = self.spawn_food();
            if self.food.is_none() {
                self.over = true;
                self.won = true;
            }
        } else {
            self.snake.pop_back();
        }
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

    fn point_to_index(&self, p: Point) -> usize {
        (p.y * self.width + p.x) as usize
    }

    fn render(&self) -> String {
        let tiles = self.build_tile_map();
        let mut out = String::new();

        out.push_str(FG_GRAY);
        out.push('+');
        for _ in 0..self.width {
            out.push_str("--");
        }
        out.push_str("+\n");

        for y in 0..self.height {
            out.push_str(FG_GRAY);
            out.push('|');
            for x in 0..self.width {
                let p = Point { x, y };
                let tile = tiles[self.point_to_index(p)];
                match tile {
                    Tile::SnakeHead => {
                        out.push_str(FG_BLUE);
                        out.push_str("██");
                    }
                    Tile::Food => {
                        out.push_str(FG_RED);
                        out.push_str("● ");
                    }
                    Tile::SnakeBody => {
                        out.push_str(FG_CYAN);
                        out.push_str("▓▓");
                    }
                    Tile::Rock => {
                        out.push_str(FG_WHITE);
                        out.push_str("◼ ");
                    }
                    Tile::Empty => {
                        if (x + y) % 2 == 0 {
                            out.push_str(BG_GRASS_A);
                        } else {
                            out.push_str(BG_GRASS_B);
                        }
                        out.push_str("  ");
                    }
                }
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
            out.push_str("Paused — press P to resume.\n");
        }

        if self.over {
            if self.won {
                out.push_str(FG_GREEN);
                out.push_str("Victory! You filled every free tile.\n");
            } else {
                out.push_str(FG_RED);
                out.push_str("Game over!\n");
            }
            out.push_str(FG_WHITE);
            out.push_str("Press Q to exit.\n");
            out.push_str(FG_GRAY);
            out.push_str("Legend: ██ head  ▓▓ body  ● food  ◼ rock\n");
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
        .find(|ch| !ch.is_whitespace())
        .map(|ch| ch.to_ascii_lowercase())
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

        if game.over {
            match rx.recv() {
                Ok(input) => {
                    if parse_command(&input) == Some('q') {
                        break;
                    }
                }
                Err(_) => break,
            }
            continue;
        }

        let tick_delay = game.speed.tick_delay();
        match rx.recv_timeout(tick_delay) {
            Ok(input) => {
                if let Some(cmd) = parse_command(&input) {
                    if game.handle_command(cmd) {
                        break;
                    }
                }

                while let Ok(queued) = rx.try_recv() {
                    if let Some(cmd) = parse_command(&queued) {
                        if game.handle_command(cmd) {
                            print!("{}", CLEAR);
                            println!(
                                "{}Thanks for playing! Final score: {}.{}",
                                FG_CYAN, game.score, RESET
                            );
                            return Ok(());
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }

        game.step();
    }

    print!("{}", CLEAR);
    println!(
        "{}Thanks for playing! Final score: {}.{}",
        FG_CYAN, game.score, RESET
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_reverse_direction() {
        let mut game = Game::new(10, 10);
        game.set_direction(Direction::Left);
        assert_eq!(game.direction, Direction::Right);
    }

    #[test]
    fn wrap_mode_crosses_edges() {
        let mut game = Game::new(6, 6);
        game.snake = VecDeque::from([Point { x: 0, y: 3 }]);
        game.direction = Direction::Left;
        game.mode = GameMode::WrapAround;
        game.step();
        assert_eq!(game.head(), Point { x: 5, y: 3 });
        assert!(!game.over);
    }

    #[test]
    fn classic_mode_hits_wall() {
        let mut game = Game::new(4, 4);
        game.mode = GameMode::Classic;
        game.step();
        game.step();
        assert!(game.over);
    }

    #[test]
    fn spawn_food_none_when_board_full() {
        let mut game = Game::new(5, 5);
        game.snake = (0..5)
            .flat_map(|y| (0..5).map(move |x| Point { x, y }))
            .collect();
        game.rocks.clear();

        assert_eq!(game.spawn_food(), None);
    }

    #[test]
    fn tiny_board_initializes_without_negative_segments() {
        let game = Game::new(1, 1);
        assert_eq!(game.snake.len(), 1);
        assert_eq!(game.head(), Point { x: 0, y: 0 });
    }

    #[test]
    fn parse_command_ignores_whitespace() {
        assert_eq!(parse_command("   D\n"), Some('d'));
    }
}
