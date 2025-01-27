mod window;
mod gpu;
use rand::Rng;
use std::{
    ops::Range,
    collections::VecDeque,
};
use window::{UserEvent, WindowId};
use winit::{event_loop::EventLoopProxy, dpi::PhysicalSize};
use abes_nice_things::prelude::*;
use std::sync::{OnceLock, Mutex};
use std::io::Read;

const SCALE: u32 = 100;
static SIZE: OnceLock<PhysicalSize<u32>> = OnceLock::new();
static HIGH_SCORE: Mutex<u32> = Mutex::new(0);

fn main() {
    get_high_score();
    let mut score = 0;
    let (temp_tx, temp_rx) = std::sync::mpsc::channel();
    let (move_tx, move_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (proxy, size) = temp_rx.recv().unwrap();
        SIZE.set(size).unwrap();
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
            debug_println!("at pos:{:?}\nGetting move...", game.snake.head);
            let attempted_move = move_rx.recv().unwrap();
            debug_println!("Got move: {attempted_move}");
            match game.valid_move(attempted_move) {
                Ok(_) => {
                    debug_println!("Move is valid: Moving");
                    game.do_move(attempted_move, &mut score)
                }
                Err(_e) => {
                    debug_println!("Invalid move: ending({_e})");
                    let high_score = HIGH_SCORE.lock().unwrap();
                    println!("Your score was: {score}");
                    if *high_score == score {
                        println!("New high score! {high_score}");
                    }
                    drop(high_score);
                    set_high_score();
                    game.proxy.send_event(UserEvent::Kill).unwrap();
                    //panic!("INTER, YOU LOSE!!11!!1!");
                    std::thread::sleep(std::time::Duration::from_secs(5));
                    panic!("Main failed to close after 5 seconds");
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
fn get_high_score() {
    if let Ok(mut file) = std::fs::File::open(".highscore") {
        let mut buf = [0_u8; 4];
        file.read_exact(&mut buf).unwrap();
        *HIGH_SCORE.lock().unwrap() = u32::from_le_bytes(buf);
    }
}
fn set_high_score() {
    std::fs::write(".highscore", (*HIGH_SCORE.lock().unwrap()).to_le_bytes()).unwrap()
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
        out.proxy.send_event(UserEvent::Move {
            pos: (out.snake.head+Dir::Left).into(),
            window: WindowId::Tail(0)
        }).unwrap();
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
            if self.snake.tail.iter().map(|x| {x.pos}).collect::<Vec<Pos>>().contains(&new_pos) {
                debug_println!("Failed because of tail intersection");
                continue;
            }
            if new_pos == self.snake.head {
                debug_println!("Failed because of head intersection");
                continue;
            }
            debug_println!("New position does not collide with tail/head");
                self.apple = new_pos;
                self.proxy.send_event(UserEvent::Move {
                    pos: new_pos.into(),
                    window: WindowId::Apple
                }).unwrap();
                return Ok(())
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
    fn do_move(&mut self, dir: Dir, score: &mut u32) {
        debug_println!("{:?}", self.snake.tail.iter());
        if self.apple == self.snake.head+dir {// grow if it ate and apple
            // self.proxy.send_event(UserEvent::ExtendTail(self.snake.head.into())).unwrap();
            *score += 1;
            let mut high_score = HIGH_SCORE.lock().unwrap();
            if *high_score < *score {
                *high_score = *score;
                debug_println!("New high score");
            }
            self.proxy.send_event(UserEvent::Move {
                pos: self.snake.head.into(),
                window: WindowId::Tail(self.snake.tail.len())
            }).unwrap();
            self.snake.move_grow(dir);
            self.randomize_apple().unwrap();
            self.proxy.send_event(UserEvent::Move {
                pos: self.snake.head.into(),
                window: WindowId::Head
            }).unwrap();
        }
        else {// otherwise, move normally without growing
            self.proxy.send_event(UserEvent::Move {
                pos: self.snake.head.into(),
                window: WindowId::Tail(self.snake.tail.back().unwrap().index)
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
            tail: VecDeque::from([TailSeg::new(head+Dir::Left,0)]),
        }
    }
    fn move_nor(&mut self, dir: Dir) {
        self.tail.back_mut().unwrap().pos = self.head;
        self.tail.rotate_right(1);
        // Moving the last tail segment to the old head pos
        self.head += dir;
    }
    fn move_grow(&mut self, dir: Dir) {
        self.tail.push_front(TailSeg::new(self.head,self.tail.len()));
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
#[derive(Debug)]
struct TailSeg {
    pos: Pos,
    index: usize
}
impl TailSeg {
    fn new(pos: Pos, index: usize) -> TailSeg {
        TailSeg {
            pos,
            index
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
impl std::ops::Sub<Dir> for Pos {
    type Output = Pos;
    fn sub(self, rhs: Dir) -> Self::Output {
        match rhs {
            Dir::Up => {
                return Pos::new(self.x, self.y-1)
            }
            Dir::Down => {
                return Pos::new(self.x, self.y+1)
            }
            Dir::Left => {
                return Pos::new(self.x+1, self.y)
            }
            Dir::Right => {
                return Pos::new(self.x-1, self.y)
            }
        }
    }
}
impl From<Pos> for winit::dpi::PhysicalPosition<u32> {
    fn from(value: Pos) -> Self {
        let size = SIZE.get().unwrap();
        winit::dpi::PhysicalPosition::new(
            (value.x*SCALE) + (size.width%SCALE),
            (value.y*SCALE) + (size.height%SCALE)
        )
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