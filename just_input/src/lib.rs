mod keyboard_state;
mod mouse_state;

pub use keyboard_state::{KeyCode, KeyboardState};
pub use mouse_state::MouseState;

use rendy::wsi::winit;

use just_core::{ecs, math, shrev};

use just_traits::scripting::{function_params::Data, ScriptApiRegistry};

use ecs::prelude::*;
use math::Vec2;
use shrev::EventChannel;
use winit::{ElementState, Event, EventsLoop, MouseButton, VirtualKeyCode, WindowEvent};

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

pub type InputChannel = EventChannel<InputEvent>;

pub struct InputSystem {}

impl InputSystem {
    pub fn initialize(world: &mut World) {
        world.resources.insert::<KeyboardState>(Default::default());
        world.resources.insert::<MouseState>(Default::default());
        world
            .resources
            .insert::<InputChannel>(InputChannel::with_capacity(64));
    }
    pub fn register_api<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let namespace = registry.register_namespace("Input", None);

        registry.register_function(
            "isKeyboardKeyPressed",
            Some(&namespace),
            |args: (Data<KeyboardState>, String)| {
                args.0.is_button_down(KeyCode::from_string(&args.1))
            },
        );

        registry.register_function(
            "keyPressedInLastFrame",
            Some(&namespace),
            |args: (Data<KeyboardState>, String)| {
                args.0
                    .button_pressed_in_last_frame(KeyCode::from_string(&args.1))
            },
        );

        registry.register_function(
            "isMouseKeyPressed",
            Some(&namespace),
            |args: (Data<MouseState>, usize)| args.0.is_button_down(args.1),
        );

        registry.register_function(
            "mousePosition",
            Some(&namespace),
            |args: Data<MouseState>| {
                let pos = args.get_mouse_position();
                Vec2::new(pos[0], pos[1])
            },
        );
    }

    pub fn poll_events(event_loop: &mut EventsLoop, world: &mut World) -> UserInput {
        let (mut keyboard_state, mut mouse_state, mut channel) =
            <(Write<KeyboardState>, Write<MouseState>, Write<InputChannel>)>::fetch(
                &mut world.resources,
            );
        let mut new_events = Vec::with_capacity(20);
        let mut output = UserInput::default();
        keyboard_state.next_frame();
        mouse_state.next_frame();
        event_loop.poll_events(|event| match event {
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
