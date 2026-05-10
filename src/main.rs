use std::{collections::VecDeque, sync::{Arc, Mutex}, thread::sleep, time::{Duration, Instant}};
use rand_set::RandSet;

const PAUSE_TIME: Duration = Duration::from_millis(200);

enum CrashError {
    Crash
}
use CrashError::Crash;

struct Snake {
    positions: VecDeque<Vec2D>
}

impl Snake {
    fn new(head: Vec2D, facing: Direction, block_count: usize) -> Self {
        let mut positions = VecDeque::from([head]);
        let backward = facing.opposite();
        for _ in 0..block_count-1 {
            let back = positions.back().unwrap();
            let new_back = back.shifted(backward);
            positions.push_back(new_back);
        }

        Self { positions }
    }

    fn extend_by(&mut self, dir: Direction) -> Vec2D {
        let prev_head = self.positions.front().unwrap();
        let new_head = prev_head.shifted(dir);
        self.positions.push_front(new_head);
        new_head
    }

    fn shrink(&mut self) {
        self.positions.pop_back();
    }

    /// Moves the snake in the direction, extending if eating a fruit.
    /// Returns true if a fruit was eaten and false otherwise.
    /// 
    /// If fruit was eaten, fruit_positions will be updated accordingly.
    fn step(&mut self, dir: Direction, fruit_positions: &mut RandSet<Vec2D>, grid_start: Vec2D, grid_end: Vec2D) -> Result<bool, CrashError> {
        let new_head = self.extend_by(dir);
        if self.positions.iter().filter(|x| **x == new_head).count() != 1 {
            return Err(Crash)
        }
        if new_head.outside(grid_start, grid_end) {
            return Err(Crash)
        }
        Ok(if !fruit_positions.remove(&new_head) {
            self.shrink();
            false
        } else {
            true
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Direction {
    Right,
    Up, 
    Left, 
    Down
}

impl Direction {
    fn opposite(self) -> Self {
        use Direction::*;
        match self {
            Right => Left,
            Up => Down,
            Left => Right,
            Down => Up,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Vec2D(isize,isize);

impl Vec2D {
    fn shifted(&self, dir: Direction) -> Self {
        let Self(x,y) = *self;
        match dir {
            Direction::Right => Self(x+1,y),
            Direction::Up => Self(x,y-1),
            Direction::Left => Self(x-1,y),
            Direction::Down => Self(x,y+1),
        }
    }

    fn outside(&self, grid_start: Vec2D, grid_end: Vec2D) -> bool {
        self.0 < grid_start.0 || self.0 > grid_end.0 || self.1 < grid_start.1 || self.1 > grid_end.1
    }
}

struct SnakeState {
    grid_start: Vec2D,
    grid_end: Vec2D,
    snake: Snake,
    current_dir: Direction,
    fruit_locations: RandSet<Vec2D>,
    complete: bool
}

impl SnakeState {
    fn new() -> Self {
        let mut out = Self {
            grid_end: Vec2D(10, 10),
            grid_start: Vec2D(0, 0),
            snake: Snake::new(Vec2D(7,5), Direction::Right, 3),
            current_dir: Direction::Right,
            fruit_locations: RandSet::new(),
            complete: false
        };
        out.add_fruit();
        out
    }

    fn free_locations(&self) -> RandSet<Vec2D> {
        let mut locations = (self.grid_start.0..self.grid_end.0)
        .flat_map(
            |x|
            (self.grid_start.1..self.grid_end.1)
            .map(move |y|Vec2D(x,y))
        )
        .collect::<RandSet<_>>();
        for location in &self.fruit_locations {
            locations.remove(location);
        }
        for location in &self.snake.positions {
            locations.remove(location);
        }
        locations
    }

    // WARNING: The following is not compatible with games with multiple fruit at a time! Update if necessary.
    /// Adds fruit if it can, returns false if it fails as there is no space, in that case, updates self.complete to true.
    fn add_fruit(&mut self) -> bool {
        let frees = self.free_locations();
        let x = frees.get_rand();
        match x {
            None => {
                self.complete = true;
                false
            },
            Some(loc) => { 
                self.fruit_locations.insert(*loc);
                true 
            }
        }
    }

    fn update(&mut self, dir: Option<Direction>) -> Result<(),CrashError> {
        let dir = match dir {
            Some(x) => x,
            None => self.current_dir
        };
        let eaten = self.snake.step(dir,&mut self.fruit_locations ,self.grid_start ,self.grid_end)?;
        if eaten {
            self.add_fruit();
        };
        Ok(())
    }
}

trait InputController {
    type Listener: Send;

    fn listener(&self) -> Self::Listener;

    fn input(&self, dur: Duration) -> Option<Direction>;
}

trait OutputHandler {
    
    fn display_frame(&mut self, game_state: &SnakeState);
}

trait IoFramework {
    type InputController: InputController;
    type OutputHandler: OutputHandler;

    fn initialise() -> (Self::InputController, Self::OutputHandler);


    // TODO: Improve game-loop as necessary. Will do for testing purposes.
    fn run_game() {
        let mut game = SnakeState::new();
        let (input_controller, mut output_handler) = Self::initialise();
        while !game.complete {
            sleep(PAUSE_TIME);
            let input = input_controller.input(PAUSE_TIME);
            game.update(input);
            output_handler.display_frame(&game);
        }
    }
}

fn main() {
    let mut game = SnakeState::new();
    let mut input_controller = InputController::new();
    while !game.complete {
        sleep(PAUSE_TIME);
        let input = input_controller.input(PAUSE_TIME);
        game.update(input);
    }
}