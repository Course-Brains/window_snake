mod window;
mod gpu;
use rand::Rng;
use std::{
    ops::Range,
    collections::VecDeque,
};
use window::{UserEvent, WindowId};
use winit::event_loop::EventLoopProxy;
use abes_nice_things::prelude::*;

const SCALE: u32 = 100;

fn main() {
    let (temp_tx, temp_rx) = std::sync::mpsc::channel();
    let (move_tx, move_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (proxy, size) = temp_rx.recv().unwrap();
        let mut game = Game::new(proxy, size);
        //std::thread::sleep(std::time::Duration::from_secs(10));
        // println!("attempting move");
        /*abes_nice_things::input();
        game.proxy.send_event(UserEvent::Move{
            pos: Pos::new(0,0).into(),
            window: WindowId::Head
        }).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));
        // println!("attempting move");
        abes_nice_things::input();
        game.proxy.send_event(UserEvent::Move{
            pos: Pos::new(7,0).into(),
            window: WindowId::Head
        }).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));
        // println!("attempting move");
        abes_nice_things::input();
        game.proxy.send_event(UserEvent::Move{
            pos: Pos::new(7,7).into(),
            window: WindowId::Head
        }).unwrap();

        std::thread::sleep(std::time::Duration::from_secs(3));
        // println!("attempting move");
        abes_nice_things::input();
        game.proxy.send_event(UserEvent::Move{
            pos: Pos::new(0,7).into(),
            window: WindowId::Head
        }).unwrap();*/
        loop {
            println!("at pos:{:?}\nGetting move...", game.snake.head);
            let attempted_move = move_rx.recv().unwrap();
            println!("Got move: {attempted_move}");
            match game.valid_move(attempted_move) {
                Ok(_) => {
                    println!("Move is valid: Moving");
                    game.do_move(attempted_move)
                }
                Err(e) => {
                    println!("Invalid move: ending({e})");
                    game.proxy.send_event(UserEvent::Kill).unwrap();
                    panic!("INTER, YOU LOSE!!11!!1!")
                }
            }
            /*game.proxy.send_event(UserEvent::Move {
                pos: Pos::new(10,0).into(),
                window: WindowId::Head
            }).unwrap();*/
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
    fn new(proxy: EventLoopProxy<UserEvent>, size: winit::dpi::PhysicalSize<u32>) -> Game {
        let mut out = Game {
            snake: Snake::new(Game::START),
            apple: Pos::new(0,0),
            proxy,
            validx: 0..(size.width/SCALE),
            validy: 0..(size.height/SCALE),
        };
        debug_println!("Creating with size: ({}, {})", out.validx.end, out.validy.end);
        out.proxy.send_event(UserEvent::Move {
            pos: Game::START.into(),
            window: WindowId::Head
        }).unwrap();
        out.proxy.send_event(UserEvent::ExtendTail((out.snake.head+Dir::Left).into())).unwrap();
        out.randomize_apple().unwrap();
        out
    }
    /// If it fails then there are no available spaces
    fn randomize_apple(&mut self) -> Result<(), ()> {
        let mut rng = rand::thread_rng();
        let mut new_pos: Pos;
        for _ in 0..((self.validx.len() * self.validy.len()) - self.snake.tail.len() + 1) {
            new_pos = Pos::new(// generating the new position
                rng.gen_range(self.validx.clone()),
                rng.gen_range(self.validy.clone())
            );
            debug_println!("Moving apple to: ({}, {})", new_pos.x, new_pos.y);
            if !self.snake.tail.iter().map(|x| {x.pos}).collect::<Vec<Pos>>().contains(&new_pos) {
                debug_println!("New position does not collide with tail");
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
    fn valid_move(&self, dir: Dir) -> Result<(), errors::InvalidMove>{
        let new_pos = self.snake.head+dir;
        if !self.validx.contains(&new_pos.x) {// is the new x on the screen
            return Err(errors::InvalidMove::OutOfBounds(new_pos))
        }
        if !self.validy.contains(&new_pos.y) {// is the new y on the screen
            return Err(errors::InvalidMove::OutOfBounds(new_pos))
        }
        if self.snake.is_tail(new_pos) {// is the new pos hitting the tail
            return Err(errors::InvalidMove::TailCollision)
        }
        return Ok(())
    }
    fn do_move(&mut self, dir: Dir) {
        if self.apple == self.snake.head+dir {// grow if it ate and apple
            self.snake.move_grow(dir);
            self.randomize_apple().unwrap();
            self.proxy.send_event(UserEvent::ExtendTail(self.snake.head.into())).unwrap();
        }
        else {// otherwise, move normally without growing
            self.proxy.send_event(UserEvent::Move {
                pos: self.snake.head.into(),
                window: WindowId::Tail(self.snake.tail.len()-1)
            }).unwrap();
            self.snake.move_nor(dir);
            self.proxy.send_event(UserEvent::Move {
                pos: self.snake.head.into(),
                window: WindowId::Head
            }).unwrap();
        }
    }
}
struct Snake {
    head: Pos,
    tail: VecDeque<TailSeg>,
}
impl Snake {
    fn new(head: Pos) -> Snake {
        Snake {
            head,
            tail: VecDeque::from([TailSeg::new(head+Dir::Left)]),
        }
    }
    fn move_nor(&mut self, dir: Dir) {
        self.tail.push_front(TailSeg::new(self.head));
        self.tail.pop_back();
        // Moving the last tail segment to the old head pos
        self.head += dir;
    }
    fn move_grow(&mut self, dir: Dir) {
        self.tail.push_front(TailSeg::new(self.head));
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
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "x: {}, y: {}", self.x, self.y)
    }
}
#[derive(Clone, Copy)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right
}
impl std::fmt::Display for Dir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dir::Up => write!(f, "Up"),
            Dir::Down => write!(f, "Down"),
            Dir::Left => write!(f, "Left"),
            Dir::Right => write!(f, "Right")
        }
    }
}
mod errors {
    use std::fmt::Display;
    pub enum InvalidMove {
        OutOfBounds(super::Pos),
        TailCollision
    }
    impl Display for InvalidMove {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                InvalidMove::OutOfBounds(pos) => {
                    write!(f, "out of bounds({pos})")
                }
                InvalidMove::TailCollision => {
                    write!(f, "tail collision")
                }
            }
        }
    }
}