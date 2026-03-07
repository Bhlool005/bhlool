use std::collections::VecDeque;
use rand::Rng;

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Game {
    pub width: i32,
    pub height: i32,
    pub snake: VecDeque<Point>,
    pub dir: Direction,
    pub food: Option<Point>,
    pub rocks: Vec<Point>,
    pub score: u32,
    pub over: bool,
    pub won: bool,
}

impl Game {
    pub fn new(width: i32, height: i32) -> Self {
        let center = Point {
            x: width / 2,
            y: height / 2,
        };

        let mut snake = VecDeque::new();
        snake.push_front(center);
        snake.push_back(Point { x: center.x - 1, y: center.y });
        snake.push_back(Point { x: center.x - 2, y: center.y });

        let mut game = Game {
            width,
            height,
            snake,
            dir: Direction::Right,
            food: None,
            rocks: Vec::new(),
            score: 0,
            over: false,
            won: false,
        };

        game.food = game.spawn_food();
        game.rocks = game.spawn_rocks(5);

        game
    }

    fn board_capacity(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn random_point(&self) -> Point {
        let mut rng = rand::thread_rng();
        Point {
            x: rng.gen_range(0..self.width),
            y: rng.gen_range(0..self.height),
        }
    }

    fn spawn_food(&self) -> Option<Point> {
        let mut attempts = 0;
        let max_attempts = self.board_capacity() * 10;

        while attempts < max_attempts {
            attempts += 1;
            let p = self.random_point();

            if !self.snake.contains(&p) {
                return Some(p);
            }
        }

        None
    }

    fn spawn_rocks(&mut self, count: usize) -> Vec<Point> {
        let mut rocks = Vec::new();
        let mut attempts = 0usize;
        let max_attempts = self.board_capacity().saturating_mul(20);

        while rocks.len() < count && attempts < max_attempts {
            attempts += 1;

            let p = self.random_point();

            if !self.snake.contains(&p) && !rocks.contains(&p) {
                rocks.push(p);
            }
        }

        rocks
    }

    pub fn change_direction(&mut self, dir: Direction) {
        match (self.dir, dir) {
            (Direction::Up, Direction::Down) => {}
            (Direction::Down, Direction::Up) => {}
            (Direction::Left, Direction::Right) => {}
            (Direction::Right, Direction::Left) => {}
            _ => self.dir = dir,
        }
    }

    pub fn update(&mut self) {
        if self.over {
            return;
        }

        let head = *self.snake.front().unwrap();

        let new_head = match self.dir {
            Direction::Up => Point { x: head.x, y: head.y - 1 },
            Direction::Down => Point { x: head.x, y: head.y + 1 },
            Direction::Left => Point { x: head.x - 1, y: head.y },
            Direction::Right => Point { x: head.x + 1, y: head.y },
        };

        if new_head.x < 0
            || new_head.y < 0
            || new_head.x >= self.width
            || new_head.y >= self.height
        {
            self.over = true;
            return;
        }

        if self.snake.contains(&new_head) {
            self.over = true;
            return;
        }

        if self.rocks.contains(&new_head) {
            self.over = true;
            return;
        }

        self.snake.push_front(new_head);

        if let Some(food) = self.food {
            if new_head == food {
                self.score += 1;
                self.food = self.spawn_food();

                if self.food.is_none() {
                    self.over = true;
                    self.won = true;
                }

                return;
            }
        }

        self.snake.pop_back();
    }
}