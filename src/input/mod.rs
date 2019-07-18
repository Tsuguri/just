use rendy::wsi::winit;
use winit::{Event, EventsLoop, VirtualKeyCode, WindowEvent, MouseButton, ElementState};
pub use keyboard_state::{KeyboardState, KeyCode};
pub use mouse_state::MouseState;

mod keyboard_state;
mod mouse_state;

#[derive(Debug, Clone, Default)]
pub struct UserInput {
    pub end_requested: bool,
    pub new_frame_size: Option<(f64, f64)>,
}

impl UserInput {
    pub fn poll_events_loop(events_loop: &mut EventsLoop, keyboard_state: &mut KeyboardState, mouse_state: &mut MouseState) -> Self {
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
            }
            Event::WindowEvent { event: WindowEvent::MouseInput { state, button,..}, .. } => {
                let id: usize = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    MouseButton::Other(oth) => oth as usize,
                };
                mouse_state.set_button_state(id, state==ElementState::Pressed);
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } => {
                //println!("pressed {:?}", input);


                if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                    output.end_requested = true;
                }
                let elem = match input.virtual_keycode {
                    Some(key) => key,
                    None => return,
                };
                keyboard_state.set_button(KeyCode::from_kc_enum(elem), input.state == winit::ElementState::Pressed);

            }
            _ => (),
        });
        output
    }
}
