use crate::input::{KeyCode, KeyboardState, MouseState};
use crate::math::{Vec2, Vec3};

use crate::traits::*;

pub struct InputApi;

impl InputApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let namespace = registry.register_namespace("Input2", None);

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
}
