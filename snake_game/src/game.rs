use std::collections::{HashSet, VecDeque};
use std::io::{self, stdout, Stdout, Write};
use std::time::{Duration, Instant};

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::style::Print;
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use rand::{thread_rng, Rng};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Point {
    x: u16,
    y: u16,
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
pub enum GameMode {
    Classic,
    WrapAround,
}

pub struct Game {
    width: u16,
    height: u16,
    mode: GameMode,
    snake: VecDeque<Point>,
    occupied: HashSet<Point>,
    direction: Direction,
    pending_direction: Direction,
    food: Point,
    score: u32,
    tick_rate: Duration,
    over: bool,
    won: bool,
}

impl Game {
    pub fn new(width: u16, height: u16, mode: GameMode) -> Self {
        let mut snake = VecDeque::new();
        let mut occupied = HashSet::new();

        let center = Point {
            x: width / 2,
            y: height / 2,
        };

        snake.push_back(center);
        snake.push_back(Point {
            x: center.x.saturating_sub(1),
            y: center.y,
        });
        snake.push_back(Point {
            x: center.x.saturating_sub(2),
            y: center.y,
        });

        occupied.extend(snake.iter().copied());

        let mut game = Self {
            width,
            height,
            mode,
            snake,
            occupied,
            direction: Direction::Right,
            pending_direction: Direction::Right,
            food: Point { x: 1, y: 1 },
            score: 0,
            tick_rate: Duration::from_millis(140),
            over: false,
            won: false,
        };
        game.food = game.spawn_food();
        game
    }

    pub fn is_over(&self) -> bool {
        self.over
    }

    fn queue_direction(&mut self, next: Direction) {
        if !self.direction.is_opposite(next) {
            self.pending_direction = next;
        }
    }

    pub fn tick_rate(&self) -> Duration {
        self.tick_rate
    }

    fn head(&self) -> Point {
        *self
            .snake
            .front()
            .expect("snake always has at least one segment")
    }

    fn spawn_food(&self) -> Point {
        let mut rng = thread_rng();
        let free_slots =
            (self.width as usize * self.height as usize).saturating_sub(self.occupied.len());
        if free_slots == 0 {
            return self.head();
        }

        let mut target = rng.gen_range(0..free_slots);
        for y in 0..self.height {
            for x in 0..self.width {
                let p = Point { x, y };
                if !self.occupied.contains(&p) {
                    if target == 0 {
                        return p;
                    }
                    target -= 1;
                }
            }
        }

        Point { x: 0, y: 0 }
    }

    fn next_head(&self, head: Point, direction: Direction) -> Option<Point> {
        match direction {
            Direction::Up => {
                if head.y == 0 {
                    (self.mode == GameMode::WrapAround).then_some(Point {
                        x: head.x,
                        y: self.height - 1,
                    })
                } else {
                    Some(Point {
                        x: head.x,
                        y: head.y - 1,
                    })
                }
            }
            Direction::Down => {
                if head.y + 1 == self.height {
                    (self.mode == GameMode::WrapAround).then_some(Point { x: head.x, y: 0 })
                } else {
                    Some(Point {
                        x: head.x,
                        y: head.y + 1,
                    })
                }
            }
            Direction::Left => {
                if head.x == 0 {
                    (self.mode == GameMode::WrapAround).then_some(Point {
                        x: self.width - 1,
                        y: head.y,
                    })
                } else {
                    Some(Point {
                        x: head.x - 1,
                        y: head.y,
                    })
                }
            }
            Direction::Right => {
                if head.x + 1 == self.width {
                    (self.mode == GameMode::WrapAround).then_some(Point { x: 0, y: head.y })
                } else {
                    Some(Point {
                        x: head.x + 1,
                        y: head.y,
                    })
                }
            }
        }
    }

    pub fn update(&mut self) {
        if self.over {
            return;
        }

        if !self.direction.is_opposite(self.pending_direction) {
            self.direction = self.pending_direction;
        }

        let Some(next_head) = self.next_head(self.head(), self.direction) else {
            self.over = true;
            return;
        };

        let tail = *self
            .snake
            .back()
            .expect("snake always has at least one segment");
        let is_growing = next_head == self.food;

        if self.occupied.contains(&next_head) && (is_growing || next_head != tail) {
            self.over = true;
            return;
        }

        self.snake.push_front(next_head);
        self.occupied.insert(next_head);

        if is_growing {
            self.score += 10;
            self.tick_rate = Duration::from_millis(
                (140u64.saturating_sub((self.score / 50) as u64 * 8)).max(55),
            );

            if self.occupied.len() == (self.width as usize * self.height as usize) {
                self.won = true;
                self.over = true;
                return;
            }

            self.food = self.spawn_food();
        } else if let Some(removed) = self.snake.pop_back() {
            self.occupied.remove(&removed);
        }
    }

    pub fn render(&self, out: &mut Stdout) -> io::Result<()> {
        queue!(out, MoveTo(0, 0), Clear(ClearType::All))?;

        // Top border
        queue!(out, Print("+"))?;
        for _ in 0..self.width {
            queue!(out, Print("-"))?;
        }
        queue!(out, Print("+\n"))?;

        for y in 0..self.height {
            queue!(out, Print("|"))?;
            for x in 0..self.width {
                let point = Point { x, y };
                let cell = if point == self.head() {
                    '@'
                } else if point == self.food {
                    '*'
                } else if self.occupied.contains(&point) {
                    'o'
                } else {
                    ' '
                };
                queue!(out, Print(cell))?;
            }
            queue!(out, Print("|\n"))?;
        }

        queue!(out, Print("+"))?;
        for _ in 0..self.width {
            queue!(out, Print("-"))?;
        }
        queue!(out, Print("+\n"))?;

        let mode = match self.mode {
            GameMode::Classic => "Classic",
            GameMode::WrapAround => "WrapAround",
        };

        queue!(
            out,
            Print(format!(
                "Score: {}  Speed: {}ms  Mode: {}\n",
                self.score,
                self.tick_rate.as_millis(),
                mode
            )),
            Print("Controls: Arrow/WASD move, P pause, M mode, Q quit\n")
        )?;

        if self.over {
            if self.won {
                queue!(out, Print("You filled the board. You win!\n"))?;
            } else {
                queue!(out, Print("Game Over!\n"))?;
            }
            queue!(out, Print("Press Q to exit.\n"))?;
        }

        out.flush()
    }

    pub fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            GameMode::Classic => GameMode::WrapAround,
            GameMode::WrapAround => GameMode::Classic,
        }
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show, LeaveAlternateScreen);
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, Hide)?;
    let _guard = TerminalGuard;

    let (term_w, term_h) = terminal::size()?;
    let board_w = term_w.saturating_sub(2).clamp(20, 60);
    let board_h = term_h.saturating_sub(8).clamp(10, 30);

    let mut out = stdout();
    let mut game = Game::new(board_w, board_h, GameMode::Classic);
    let mut last_tick = Instant::now();
    let mut paused = false;

    loop {
        let timeout = game
            .tick_rate()
            .saturating_sub(last_tick.elapsed())
            .min(Duration::from_millis(250));

        if event::poll(timeout)? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                match key_event.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Up | KeyCode::Char('w') => game.queue_direction(Direction::Up),
                    KeyCode::Down | KeyCode::Char('s') => game.queue_direction(Direction::Down),
                    KeyCode::Left | KeyCode::Char('a') => game.queue_direction(Direction::Left),
                    KeyCode::Right | KeyCode::Char('d') => game.queue_direction(Direction::Right),
                    KeyCode::Char('p') => paused = !paused,
                    KeyCode::Char('m') if !game.is_over() => game.toggle_mode(),
                    _ => {}
                }
            }
        }

        if !paused && last_tick.elapsed() >= game.tick_rate() {
            game.update();
            last_tick = Instant::now();
        }

        game.render(&mut out)?;

        if paused {
            queue!(out, Print("Paused\n"))?;
            out.flush()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cannot_reverse_direction_instantly() {
        let mut game = Game::new(12, 12, GameMode::Classic);
        game.queue_direction(Direction::Left);
        game.update();
        assert_eq!(game.direction, Direction::Right);
    }

    #[test]
    fn wrap_mode_allows_crossing_edges() {
        let mut game = Game::new(6, 6, GameMode::WrapAround);
        game.snake = VecDeque::from([Point { x: 0, y: 3 }]);
        game.occupied = HashSet::from([Point { x: 0, y: 3 }]);
        game.direction = Direction::Left;
        game.pending_direction = Direction::Left;
        game.food = Point { x: 2, y: 2 };

        game.update();

        assert_eq!(game.head(), Point { x: 5, y: 3 });
        assert!(!game.over);
    }
}
