mod window;
mod gpu;
use rand::Rng;
use std::{
    ops::Range,
    collections::VecDeque,
};
use window::UserEvent;
use winit::event_loop::EventLoopProxy;

const SCALE: usize = 100;

fn main() {
    Game::new();
    std::thread::sleep(std::time::Duration::from_secs(20));
}
struct Game {
    snake: Snake,
    apple: Pos,
    apple_window: EventLoopProxy<UserEvent>,
    validx: Range<u32>,
    validy: Range<u32>
}
impl Game {
    const START: Pos = Pos::new(4, 7);
    fn new() -> Game {
        let mut out = Game {
            snake: Snake::new(Game::START),
            apple: Pos::new(0,0),
            apple_window: window::new(),
            validx: 0..10,
            validy: 0..10,
        };
        out.randomize_apple().unwrap();
        out.apple_window.send_event(UserEvent::RED);
        out
    }
    // If it fails then there are no available spaces
    fn randomize_apple(&mut self) -> Result<(), ()> {
        let mut rng = rand::thread_rng();
        let mut new_pos: Pos;
        for _ in 0..((self.validx.len() * self.validy.len()) - self.snake.tail.len() + 1) {
            new_pos = Pos::new(
                rng.gen_range(self.validx.clone()),
                rng.gen_range(self.validy.clone())
            );
            if self.snake.tail.iter().map(|x| {x.pos}).collect::<Vec<Pos>>().contains(&new_pos) {
                self.apple = new_pos;
                self.apple_window.send_event(UserEvent::Move(new_pos.into())).unwrap();
                return Ok(())
            }
        }
        return Err(())
    }
}
struct Snake {
    head: Pos,
    head_window: EventLoopProxy<UserEvent>,
    tail: VecDeque<TailSeg>,
    next_window: EventLoopProxy<UserEvent>
}
impl Snake {
    fn new(head: Pos) -> Snake {
        Snake {
            head,
            head_window: window::new(),
            tail: VecDeque::new(),
            next_window: window::new(),
        }
    }
    fn col_check(&self) -> Result<(), Pos> {
        for seg in self.tail.iter() {
            if seg.pos == self.head {
                return Err(seg.pos)
            }
        }
        Ok(())
    }
    fn move_nor(&mut self, dir: Dir) {
        let window = self.tail.pop_back().unwrap().window;
        self.tail.push_front(TailSeg::new(self.head, window));
        self.head += dir;
    }
    fn move_grow(&mut self, dir: Dir) {
        self.tail.push_front(TailSeg::new(self.head, self.next_window.clone()));
        self.next_window = window::new();
        self.head += dir;
    }
}
struct TailSeg {
    pos: Pos,
    window: EventLoopProxy<UserEvent>
}
impl TailSeg {
    fn new(pos: Pos, window: EventLoopProxy<UserEvent>) -> TailSeg {
        TailSeg {
            pos,
            window
        }
    }
}
impl PartialEq<Pos> for TailSeg {
    fn eq(&self, other: &Pos) -> bool {
        self.pos == *other
    }
}
#[derive(PartialEq, Eq, Clone, Copy)]
struct Pos {
    x: u32,
    y: u32
}
impl Pos {
    const fn new(x: u32, y: u32) -> Pos {
        Pos {
            x,
            y
        }
    }
}
impl std::ops::Add<Dir> for Pos {
    type Output = Pos;
    fn add(self, rhs: Dir) -> Self::Output {
        match rhs {
            Dir::Up => {
                return Pos::new(self.x, self.y+1)
            },
            Dir::Down => {
                return Pos::new(self.x, self.y-1)
            },
            Dir::Left => {
                return Pos::new(self.x-1, self.y)
            },
            Dir::Right => {
                return Pos::new(self.x+1, self.y)
            }
        }
    }
}
impl std::ops::AddAssign<Dir> for Pos {
    fn add_assign(&mut self, rhs: Dir) {
        match rhs { 
            Dir::Up => {
                self.y += 1;
            },
            Dir::Down => {
                self.y -= 1;
            },
            Dir::Left => {
                self.x -= 1;
            },
            Dir::Right => {
                self.x += 1;
            }
        }
    }
}
impl From<Pos> for winit::dpi::PhysicalPosition<u32> {
    fn from(value: Pos) -> Self {
        winit::dpi::PhysicalPosition::new(value.x, value.y)
    }
}
enum Dir {
    Up,
    Down,
    Left,
    Right
}