mod window;
mod gpu;
use rand::Rng;
use std::{
    ops::Range,
    collections::VecDeque,
};
use window::{UserEvent, WindowId};
use winit::event_loop::{EventLoop, EventLoopProxy, EventLoopBuilder};

const SCALE: u32 = 1;

fn main() {
    let (temp_tx, temp_rx) = std::sync::mpsc::channel();
    let (move_tx, move_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut game = Game::new(temp_rx.recv().unwrap());
        loop {
            let attempted_move = move_rx.recv().unwrap();
            if game.valid_move(attempted_move) {
                game.do_move(attempted_move)
            }
            else {
                game.proxy.send_event(UserEvent::Kill).unwrap();
                panic!("INTER, YOU LOSE!!11!!1!")
            }
        }
    });
    window::run(temp_tx, move_tx);
}
struct Game {
    snake: Snake,
    apple: Pos,
    proxy: EventLoopProxy<UserEvent>,
    validx: Range<u32>,
    validy: Range<u32>
}
impl Game {
    const START: Pos = Pos::new(4, 7);
    fn new(proxy: EventLoopProxy<UserEvent>) -> Game {
        let mut out = Game {
            snake: Snake::new(Game::START, proxy.clone()),
            apple: Pos::new(0,0),
            proxy,
            validx: 0..10,
            validy: 0..10,
        };
        out.proxy.send_event(UserEvent::Move {
            pos: Game::START.into(),
            window: WindowId::Head
        }).unwrap();
        out.randomize_apple().unwrap();
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
                self.proxy.send_event(UserEvent::Move {
                    pos: new_pos.into(),
                    window: WindowId::Apple
                }).unwrap();
                return Ok(())
            }
        }
        return Err(())
    }
    fn valid_move(&self, dir: Dir) -> bool{
        let new_pos = self.snake.head+dir;
        if self.validx.contains(&new_pos.x) {// is the new x on the screen
            return false
        }
        if self.validy.contains(&new_pos.y) {// is the new y on the screen
            return false
        }
        if self.snake.is_tail(new_pos) {// is the new pos hitting the tail
            return false
        }
        return true
    }
    fn do_move(&mut self, dir: Dir) {
        if self.apple == self.snake.head+dir {// grow if it ate and apple
            self.snake.move_grow(dir)
        }
        else {// otherwise, move normally without growing
            self.snake.move_nor(dir)
        }
    }
}
struct Snake {
    head: Pos,
    tail: VecDeque<TailSeg>,
    proxy: EventLoopProxy<UserEvent>
}
impl Snake {
    fn new(head: Pos, proxy: EventLoopProxy<UserEvent>) -> Snake {
        Snake {
            head,
            tail: VecDeque::new(),
            proxy
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
        self.tail.push_front(TailSeg::new(self.head));
        self.tail.pop_back();
        // Moving the last tail segment to the old head pos
        self.proxy.send_event(UserEvent::Move {
            pos: self.head.into(),
            window: WindowId::Tail(self.tail.len()-1)
        }).unwrap();
        self.head += dir;
    }
    fn move_grow(&mut self, dir: Dir) {
        self.tail.push_front(TailSeg::new(self.head));
        self.proxy.send_event(UserEvent::ExtendTail(self.head.into())).unwrap();
        self.head += dir;
    }
    /// Returns true if it is a point on the tail
    fn is_tail(&self, pos: Pos) -> bool {
        for tail_seg in self.tail.iter() {
            if tail_seg.pos == pos {
                return true
            }
        }
        false
    }
}
struct TailSeg {
    pos: Pos,
}
impl TailSeg {
    fn new(pos: Pos) -> TailSeg {
        TailSeg {
            pos,
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
        winit::dpi::PhysicalPosition::new(value.x*SCALE, value.y*SCALE)
    }
}
#[derive(Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right
}