use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::WindowBuilder,
    dpi::PhysicalPosition
};
use std::sync::mpsc;
use crate::gpu::State;

#[derive(Debug)]
pub enum UserEvent {
    Move(PhysicalPosition<u32>),
    Color(wgpu::Color),
    Visible(bool),
    Close,
}
impl UserEvent {
    pub const RED: UserEvent = UserEvent::Color(wgpu::Color::RED);
    pub const BLUE: UserEvent = UserEvent::Color(wgpu::Color::BLUE);
    pub const GREEN: UserEvent = UserEvent::Color(wgpu::Color::GREEN);
    pub const VISIBLE: UserEvent = UserEvent::Visible(true);
    pub const INVISIBLE: UserEvent = UserEvent::Visible(false);
}

pub fn new() -> EventLoopProxy<UserEvent> {
    env_logger::init();
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let event_loop = winit::event_loop::EventLoopBuilder::with_user_event().build();
        tx.send(event_loop.create_proxy()).unwrap();
        pollster::block_on(new_async(event_loop));
    });
    return rx.recv().unwrap()
}
async fn new_async(event_loop: EventLoop<UserEvent>) {
    let window = WindowBuilder::new()
        .build(&event_loop)
    .unwrap();
    window.set_resizable(false);
    window.set_decorations(false);

    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_poll();
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                    },
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                        state.window.request_redraw();
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                        state.window.request_redraw()
                    }
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if lost
                    Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            Event::UserEvent(user_event) => {
                match user_event {
                    UserEvent::Move(pos) => {
                        state.window.set_outer_position(pos)
                    }
                    UserEvent::Color(color) => {
                        state.color = color;
                    }
                    UserEvent::Visible(visible) => {
                        state.window.set_visible(visible);
                    }
                    UserEvent::Close => {
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            _ => {}
        }
    });
}