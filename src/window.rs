use pollster::FutureExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopBuilder, EventLoopWindowTarget},
    window::{WindowBuilder, Window},
    dpi::PhysicalPosition
};
use std::{
    collections::HashMap,
    sync::mpsc::Sender
};
use crate::gpu::State;
use abes_nice_things::prelude::*;
const SIZE: winit::dpi::PhysicalSize<u32> = winit::dpi::PhysicalSize::new(crate::SCALE, crate::SCALE);

pub struct Windows {
    head: State,
    apple: State,
    tail_segs: Vec<State>,
    lookup: HashMap<winit::window::WindowId, WindowId>,
}
impl Windows {
    pub fn new(event_loop: &EventLoopWindowTarget<UserEvent>) -> Result<Windows, winit::error::OsError> {
        let mut out = Windows {
            head: State::new(gen_window(event_loop)).block_on(),
            apple: State::new(gen_window(event_loop)).block_on(),
            tail_segs: Vec::new(),
            lookup: HashMap::new(),
        };
        let total = {
            let size = out.head.window.current_monitor().unwrap().size();
            (size.width/crate::SCALE)*(size.height/crate::SCALE)
        };
        debug_println!("Creating {total} tail segments");
        out.tail_segs.reserve_exact(total as usize - 1);
        let mut new: State;
        for index in 0..total {
            debug_println!("Adding tail segment#{index}");
            new = State::new(
                WindowBuilder::new()
                .with_active(false)
                .with_decorations(false)
                .with_enabled_buttons(winit::window::WindowButtons::empty())
                .with_inner_size(SIZE)
                .with_position(winit::dpi::PhysicalPosition::new(-(crate::SCALE as i32 *4), -(crate::SCALE as i32 *4)))
                .build(event_loop).unwrap()
            ).block_on();
            new.color = wgpu::Color::GREEN;
            out.lookup.insert(new.window.id(), WindowId::Tail(index as usize));
            out.tail_segs.push(new);
        }
        // Initialization of colors/visiblity
        out.head.color = wgpu::Color::BLUE;
        out.apple.color = wgpu::Color::RED;
        // Rendering colors
        out.head.render().unwrap();
        out.apple.render().unwrap();
        // setting the head to be the active window
        out.head.window.focus_window();
        // Setting up the lookup
        out.lookup.insert(out.head.window.id(), WindowId::Head);
        out.lookup.insert(out.apple.window.id(), WindowId::Apple);
        //out.lookup.insert(out.next.window.id(), WindowId::Next);
        return Ok(out);
    }
    fn add_tail_seg(&mut self, _pos: PhysicalPosition<u32>, _event_loop: &EventLoopWindowTarget<UserEvent>) {
        /*println!("Moving next tail segment to @({},{})",pos.x,pos.y);
        let next = self.next.pop().expect("We had a little fucky wucky");
        next.window.set_outer_position(pos);


        let mut new = State::new(gen_window(event_loop)).block_on();
        new.window.set_outer_position(pos);
        new.color = wgpu::Color::GREEN;
        self.lookup.insert(new.window.id(), WindowId::Tail(self.tail_segs.len()));
        self.tail_segs.push(new);*/
    }
    fn change_state(&mut self, id: WindowId, mut method: impl FnMut(&mut State)) {
        match id {
            WindowId::Head => {
                method(&mut self.head);
            }
            WindowId::Apple => {
                method(&mut self.apple);
            }
            /*WindowId::Next => {
                method(&mut self.next);
            }*/
            WindowId::Tail(index) => {
                method(&mut self.tail_segs[index]);
            }
        }
    }
}
#[derive(Debug)]
pub enum UserEvent {
    Move{
        pos: PhysicalPosition<u32>,
        window: WindowId
    },
    ExtendTail(PhysicalPosition<u32>),
    //Redraw(WindowId),
    Kill,
}
#[derive(Debug, Copy, Clone)]
pub enum WindowId {
    Head,
    Apple,
    Tail(usize),
}
impl std::fmt::Display for WindowId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WindowId::Head => write!(f, "head"),
            WindowId::Apple => write!(f, "apple"),
            WindowId::Tail(index) => write!(f, "tail#{index}")
        }
    }
}
fn gen_window(window_target: &EventLoopWindowTarget<UserEvent>) -> Window {
    WindowBuilder::new()
        .with_active(false)
        .with_decorations(false)
        .with_enabled_buttons(winit::window::WindowButtons::empty())
        .with_resizable(false)
        .with_inner_size(SIZE)
    .build(window_target).unwrap()
}
type Setup = (EventLoopProxy<UserEvent>, winit::dpi::PhysicalSize<u32>);
/// Will block the current thread until the event loop exits
pub fn run(proxy_sender: Sender<Setup>, move_sender: Sender<crate::Dir>) {
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let mut windows = Windows::new(&event_loop).unwrap();
    proxy_sender.send(
        (
            event_loop.create_proxy(),
            windows.head.window.current_monitor().unwrap().size()
        )
    ).unwrap();

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_wait();
        // println!("{:?}",event);
        match event {
            Event::WindowEvent {
                ref event,
                ..
            } => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::KeyboardInput {
                        input, ..
                    } => {
                        match input {
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            } => {
                                debug_println!("Quitting");
                                *control_flow = ControlFlow::Exit;
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..// W needs to go down because y has the top as 0
                            }| KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Down),
                                ..
                            } => {
                                debug_println!("Moving up");
                                move_sender.send(crate::Dir::Up).unwrap()
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            }| KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Left),
                                ..
                            } => {
                                debug_println!("Moving left");
                                move_sender.send(crate::Dir::Left).unwrap()
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            }| KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Up),
                                ..
                            } => {
                                debug_println!("Moving down");
                                move_sender.send(crate::Dir::Down).unwrap()
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::D),
                                ..
                            }| KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Right),
                                ..
                            } => {
                                debug_println!("Moving right");
                                move_sender.send(crate::Dir::Right).unwrap()
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) => {
                let id = *windows.lookup.get(&window_id).unwrap();
                debug_println!("redrawing {id}");
                windows.change_state(id, |state| {
                    match state.render() {
                        Ok(_) => {}
                        // Reconfigure the surface if lost
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        // The system is out of memory, we should probably quit
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // All other errors (Outdated, Timeout) should be resolved by the next frame
                        Err(e) => eprintln!("{:?}", e),
                    }
                });
            }
            Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::Move{pos, window} => {
                        windows.change_state(window, |state| {
                            debug_println!("move");
                            state.window.set_outer_position(pos);
                            debug_println!("move done")
                        })
                    }
                    UserEvent::ExtendTail(pos) => {
                        debug_println!("extend");
                        windows.add_tail_seg(pos, window_target);
                        debug_println!("extend done");
                    }
                    /*UserEvent::Redraw(window_id) => {
                        debug_println!("redraw");
                        windows.change_state(window_id, |state| {
                            state.window.request_redraw();
                        });
                        debug_println!("redraw done");
                    }*/
                    UserEvent::Kill => {
                        debug_println!("KILL");
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            _ => {}
        }
    });
}