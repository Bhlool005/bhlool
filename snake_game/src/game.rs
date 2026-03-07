use rand::Rng;
use crossterm::style::{Color, SetForegroundColor, ResetColor};

#[derive(PartialEq, Clone, Copy)]
pub enum Direction { Up, Down, Left, Right }

pub struct Snake {
    pub body: Vec<(i32,i32)>,
    pub direction: Direction,
    pub alive: bool,
}

pub struct Game {
    pub snake: Snake,
    pub ai_snakes: Vec<Snake>,
    pub food: Vec<(i32,i32)>,
    pub width: i32,
    pub height: i32,
    pub game_over: bool,
    pub score: i32,
    pub speed: u64,
}

impl Game {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();

        // Player snake
        let snake = Snake {
            body: vec![(25,12),(24,12),(23,12)],
            direction: Direction::Right,
            alive: true,
        };

        // AI snakes
        let mut ai_snakes = vec![];
        for _ in 0..3 {
            ai_snakes.push(Snake {
                body: vec![(rng.gen_range(5..45), rng.gen_range(5..20))],
                direction: match rng.gen_range(0..4) {
                    0=>Direction::Up,1=>Direction::Down,2=>Direction::Left,_=>Direction::Right
                },
                alive: true
            });
        }

        // Food
        let mut food = vec![];
        for _ in 0..7 {
            food.push((rng.gen_range(1..49), rng.gen_range(1..24)));
        }

        Game {
            snake,
            ai_snakes,
            food,
            width: 50,
            height: 25,
            game_over: false,
            score: 0,
            speed: 120
        }
    }

    pub fn change_direction(&mut self, dir:Direction) {
        let current = self.snake.direction;
        if (current==Direction::Up && dir==Direction::Down) ||
           (current==Direction::Down && dir==Direction::Up) ||
           (current==Direction::Left && dir==Direction::Right) ||
           (current==Direction::Right && dir==Direction::Left) { return; }
        self.snake.direction = dir;
    }

    pub fn update(&mut self) {
        // Move player snake
        let (mut x, mut y) = self.snake.body[0];
        match self.snake.direction {
            Direction::Up => y-=1,
            Direction::Down => y+=1,
            Direction::Left => x-=1,
            Direction::Right => x+=1
        }

        if x<=0 || x>=self.width-1 || y<=0 || y>=self.height-1 || self.snake.body.contains(&(x,y)) {
            self.game_over = true;
            return;
        }

        for ai in &self.ai_snakes {
            if ai.alive && ai.body.contains(&(x,y)) { self.game_over=true; return; }
        }

        self.snake.body.insert(0,(x,y));

        if let Some(pos) = self.food.iter().position(|&f| f==(x,y)) {
            self.score += 1;
            if self.speed > 40 { self.speed -= 2; }
            let mut rng = rand::thread_rng();
            self.food[pos] = (rng.gen_range(1..self.width-1), rng.gen_range(1..self.height-1));
        } else {
            self.snake.body.pop();
        }
    }

    pub fn update_ai(&mut self) {
        let mut rng = rand::thread_rng();
        for ai in &mut self.ai_snakes {
            if !ai.alive { 
                // Respawn AI snake
                ai.body = vec![(rng.gen_range(5..45), rng.gen_range(5..20))];
                ai.direction = match rng.gen_range(0..4) {
                    0=>Direction::Up,1=>Direction::Down,2=>Direction::Left,_=>Direction::Right
                };
                ai.alive = true;
            }
            let (mut x, mut y) = ai.body[0];
            // AI tries to move toward nearest food sometimes
            if rng.gen_bool(0.5) {
                let target = self.food[rng.gen_range(0..self.food.len())];
                if x < target.0 { ai.direction=Direction::Right; }
                else if x > target.0 { ai.direction=Direction::Left; }
                else if y < target.1 { ai.direction=Direction::Down; }
                else if y > target.1 { ai.direction=Direction::Up; }
            } else if rng.gen_bool(0.1) {
                ai.direction = match rng.gen_range(0..4) {
                    0=>Direction::Up,1=>Direction::Down,2=>Direction::Left,_=>Direction::Right
                };
            }
            match ai.direction {
                Direction::Up => y-=1,
                Direction::Down => y+=1,
                Direction::Left => x-=1,
                Direction::Right => x+=1
            }
            if x<=0 || x>=self.width-1 || y<=0 || y>=self.height-1 {
                ai.alive=false; continue;
            }
            ai.body.insert(0,(x,y));
            ai.body.pop();
        }
    }

    pub fn check_collisions(&mut self) {
        // Player collides with AI snakes
        for ai in &mut self.ai_snakes {
            if !ai.alive { continue; }
            for part in &self.snake.body {
                if ai.body.contains(part) {
                    self.game_over=true;
                    return;
                }
            }
        }
    }

    pub fn render(&self) {
        print!("\x1B[2J\x1B[1;1H");
        println!("Score: {}", self.score);

        for y in 0..self.height {
            for x in 0..self.width {
                if x==0 || x==self.width-1 || y==0 || y==self.height-1 {
                    print!("{}", SetForegroundColor(Color::Blue)); print!("▓"); print!("{}", ResetColor);
                } else if self.food.contains(&(x,y)) {
                    print!("{}", SetForegroundColor(Color::Red)); print!("●"); print!("{}", ResetColor);
                } else if self.snake.body[0]==(x,y) {
                    print!("{}", SetForegroundColor(Color::Yellow)); print!("■"); print!("{}", ResetColor);
                } else if self.snake.body.contains(&(x,y)) {
                    print!("{}", SetForegroundColor(Color::Green)); print!("■"); print!("{}", ResetColor);
                } else {
                    let mut drawn=false;
                    for ai in &self.ai_snakes {
                        if !ai.alive { continue; }
                        if ai.body[0]==(x,y) { print!("{}", SetForegroundColor(Color::Magenta)); print!("■"); print!("{}", ResetColor); drawn=true; break; }
                        else if ai.body.contains(&(x,y)) { print!("{}", SetForegroundColor(Color::DarkMagenta)); print!("■"); print!("{}", ResetColor); drawn=true; break; }
                    }
                    if !drawn { print!("{}", SetForegroundColor(Color::DarkGrey)); print!("·"); print!("{}", ResetColor); }
                }
            }
            println!();
        }
    }
}