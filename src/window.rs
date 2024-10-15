use pollster::FutureExt;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy, EventLoopBuilder, EventLoopWindowTarget},
    window::{WindowBuilder, Window},
    dpi::PhysicalPosition
};
use std::{
    collections::HashMap,
    ops::Deref,
    sync::mpsc::Sender
};
use crate::gpu::State;
use abes_nice_things::prelude::*;

pub struct Windows {
    head: State,
    apple: State,
    tail_segs: Vec<State>,
    //next: ThreadInit<State>,
    lookup: HashMap<winit::window::WindowId, WindowId>,
}
impl Windows {
    pub fn new(event_loop: &EventLoopWindowTarget<UserEvent>) -> Result<Windows, winit::error::OsError> {
        //let window_target: &EventLoopWindowTarget<UserEvent> = event_loop.deref();
        let mut out = Windows {
            head: State::new(gen_window(event_loop)).block_on(),
            apple: State::new(gen_window(event_loop)).block_on(),
            tail_segs: Vec::new(),
            /*next: ThreadInit::new(&|| {
                let out: State = State::new(
                    WindowBuilder::new()
                    .with_visible(false)
                    .with_active(false)
                    .with_decorations(false)
                    .with_enabled_buttons(winit::window::WindowButtons::empty())
                    .build(window_target).unwrap()
                ).block_on();
                out.color = wgpu::Color::GREEN;
                out.render();
                return out;
            })
            State::new(
                WindowBuilder::new()
                    .with_active(false)
                    .with_visible(false)
                    .with_decorations(false)
                    .with_enabled_buttons(winit::window::WindowButtons::empty())
                    .with_resizable(false)
                .build(event_loop).unwrap()
            ).block_on(),*/
            lookup: HashMap::new(),
        };
        // Initialization of colors/visiblity
        out.head.color = wgpu::Color::BLUE;
        out.apple.color = wgpu::Color::RED;
        //out.next.color = wgpu::Color::GREEN;
        // Rendering colors
        out.head.render().unwrap();
        out.apple.render().unwrap();
        //out.next.render().unwrap();
        // setting the head to be the active window
        out.head.window.focus_window();
        // Setting up the lookup
        out.lookup.insert(out.head.window.id(), WindowId::Head);
        out.lookup.insert(out.apple.window.id(), WindowId::Apple);
        //out.lookup.insert(out.next.window.id(), WindowId::Next);
        return Ok(out);
    }
    fn add_tail_seg(&mut self, pos: PhysicalPosition<u32>, event_loop: &EventLoopWindowTarget<UserEvent>) {
        /*let window_target = event_loop.deref();
        let new_next = ThreadInit::new(&|| {
            let mut out = State::new(
                WindowBuilder::new()
                    .with_visible(false)
                    .with_active(false)
                    .with_decorations(false)
                    .with_enabled_buttons(winit::window::WindowButtons::empty())
                .build(window_target).unwrap()
            ).block_on();
            out.color = wgpu::Color::GREEN;
            out.render();
            return out;
        });*/
        /*let mut new_next = State::new(
            WindowBuilder::new()
                .with_active(false)
                .with_visible(false)
                .with_decorations(false)
                .with_enabled_buttons(winit::window::WindowButtons::empty())
                .with_resizable(false)
                .with_position(pos)
            .build(event_loop).unwrap()
        ).block_on();
        new_next.color = wgpu::Color::GREEN;
        new_next.render().unwrap();
        let new = std::mem::replace(&mut self.next, new_next);*/
        let new = State::new(
            WindowBuilder::new()
                .with_active(false)
                .with_decorations(false)
                .with_enabled_buttons(winit::window::WindowButtons::empty())
                .with_resizable(false)
                .with_position(pos)
            .build(event_loop).unwrap()
        ).block_on();
        self.lookup.insert(new.window.id(), WindowId::Tail(self.tail_segs.len()));
        self.tail_segs.push(new);
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
    Color{
        color: wgpu::Color,
        window: WindowId
    },
    Visible{
        visible: bool,
        window: WindowId
    },
    ExtendTail(PhysicalPosition<u32>),
    Kill,
}
impl UserEvent {
    pub const RED: wgpu::Color = (wgpu::Color::RED);
    pub const BLUE: wgpu::Color = (wgpu::Color::BLUE);
    pub const GREEN: wgpu::Color = (wgpu::Color::GREEN);
}
#[derive(Debug, Copy, Clone)]
pub enum WindowId {
    Head,
    Apple,
    //Next,
    Tail(usize),
}
fn gen_window(window_target: &EventLoopWindowTarget<UserEvent>) -> Window {
    WindowBuilder::new()
        .with_active(false)
        .with_decorations(false)
        .with_enabled_buttons(winit::window::WindowButtons::empty())
        .with_resizable(false)
    .build(window_target).unwrap()
}
/// Will block the current thread until the event loop ends
pub fn run(proxy_sender: Sender<EventLoopProxy<UserEvent>>, move_sender: Sender<crate::Dir>) {
    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    let mut windows = Windows::new(&event_loop).unwrap();
    proxy_sender.send(event_loop.create_proxy()).unwrap();

    event_loop.run(move |event, window_target, control_flow| {
        control_flow.set_wait();
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
                        debug_println!("{:?}", event);
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
                                virtual_keycode: Some(VirtualKeyCode::W),
                                ..
                            } => {
                                debug_println!("Moving up");
                                move_sender.send(crate::Dir::Up).unwrap()
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::A),
                                ..
                            } => {
                                debug_println!("Moving left");
                                move_sender.send(crate::Dir::Left).unwrap()
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::S),
                                ..
                            } => {
                                debug_println!("Moving down");
                                move_sender.send(crate::Dir::Down).unwrap()
                            }
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::D),
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
                            state.window.set_outer_position(pos)
                        })
                    }
                    UserEvent::Color{color, window} => {
                        windows.change_state(window, |state| {
                            state.color = color
                        })
                    }
                    UserEvent::Visible{visible, window} => {
                        windows.change_state(window, |state| {
                            state.window.set_visible(visible)
                        })
                    }
                    UserEvent::ExtendTail(pos) => {
                        windows.add_tail_seg(pos, window_target);
                    }
                    UserEvent::Kill => {
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            _ => {}
        }
    });
}