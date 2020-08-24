use crate::math::Vec2;
pub use keyboard_state::{KeyCode, KeyboardState};
use legion::prelude::*;
pub use mouse_state::MouseState;
use rendy::wsi::winit;
use shrev::EventChannel;
use winit::{ElementState, Event, EventsLoop, MouseButton, VirtualKeyCode, WindowEvent};

mod keyboard_state;
mod mouse_state;

#[derive(Debug, Clone, Default)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<(f64, f64)>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum InputEvent {
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    MouseButtonPressed(usize),
    MouseButtonReleased(usize),
    MouseMoved(Vec2),
}

impl UserInput {
    pub fn initialize(world: &mut World) {
        world.resources.insert::<KeyboardState>(Default::default());
        world.resources.insert::<MouseState>(Default::default());
        world
            .resources
            .insert::<EventChannel<InputEvent>>(EventChannel::with_capacity(64));
    }

    pub fn poll_events_loop(events_loop: &mut EventsLoop, world: &mut World) -> Self {
        let (mut keyboard_state, mut mouse_state, mut channel) =
            <(
                Write<KeyboardState>,
                Write<MouseState>,
                Write<EventChannel<InputEvent>>,
            )>::fetch(&mut world.resources);
        let mut new_events = Vec::with_capacity(20);
        let mut output = UserInput::default();
        keyboard_state.next_frame();
        mouse_state.next_frame();
        events_loop.poll_events(|event| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => output.end_requested = true,
            Event::WindowEvent {
                event: WindowEvent::Resized(logical),
                ..
            } => {
                output.new_frame_size = Some((logical.width, logical.height));
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                mouse_state.set_new_position([position.x as f32, position.y as f32]);
                new_events.push(InputEvent::MouseMoved(Vec2::new(
                    position.x as f32,
                    position.y as f32,
                )));
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                let id: usize = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Other(oth) => oth as usize,
                };
                mouse_state.set_button_state(id, state == ElementState::Pressed);
                match state {
                    ElementState::Pressed => new_events.push(InputEvent::MouseButtonPressed(id)),
                    ElementState::Released => new_events.push(InputEvent::MouseButtonReleased(id)),
                }
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                //println!("pressed {:?}", input);

                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                    output.end_requested = true;
                }
                let elem = match input.virtual_keycode {
                    Some(key) => key,
                    None => return,
                };
                let kc = KeyCode::from_kc_enum(elem);
                keyboard_state.set_button(kc, input.state == winit::ElementState::Pressed);
                match input.state {
                    ElementState::Pressed => new_events.push(InputEvent::KeyPressed(kc)),
                    ElementState::Released => new_events.push(InputEvent::KeyReleased(kc)),
                }
            }
            _ => (),
        });
        channel.drain_vec_write(&mut new_events);
        output
    }
}