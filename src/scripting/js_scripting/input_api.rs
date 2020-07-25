use super::js;

use js::{
    ContextGuard,
    value::function::CallbackInfo,
    value::Value,
};

use super::api_helpers::*;
use crate::input::{KeyCode, KeyboardState, MouseState};
use crate::scripting::InternalTypes;
use crate::math::Vec3;

use crate::traits::*;

fn keyboard_is_key_pressed(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value>{
    let ctx = guard.context();
    let world = world(&ctx);
    let keyboard = world.resources.get::<KeyboardState>().unwrap();
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].to_string(guard);

    let result = keyboard.is_button_down(KeyCode::from_string(&arg));
    Result::Ok(js::value::Boolean::new(guard, result).into())
}

fn keyboard_was_pressed_in_last_frame(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let keyboard = world.resources.get::<KeyboardState>().unwrap();
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].to_string(guard);

    let result = keyboard.button_pressed_in_last_frame(KeyCode::from_string(&arg));
    Result::Ok(js::value::Boolean::new(guard, result).into())

}

fn mouse_is_key_pressed(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let mouse = world.resources.get::<MouseState>().unwrap();
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].clone().into_number().unwrap();
    let arg_value = arg.value();

    let result = mouse.is_button_down(arg_value as usize);
    Result::Ok(js::value::Boolean::new(guard, result).into())
}

fn mouse_position(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let mouse = world.resources.get::<MouseState>().unwrap();
    let pos = mouse.get_mouse_position();
    let prototypes = prototypes(&ctx);
    
    let obj = js::value::External::new(guard, Box::new(Vec3::new(pos[0], pos[1], 0.0f32)));
    obj.set_prototype(guard, prototypes[&InternalTypes::Vec3].clone()).unwrap();
    Result::Ok(obj.into())

}

impl super::JsScriptEngine {
    pub fn create_input_api(&mut self) {
        let module = self.create_api_module("Input");
        let guard = self.guard();

        add_function(&guard, &module, "isKeyboardKeyPressed", Box::new(|a,b| keyboard_is_key_pressed(a,b)));
        add_function(&guard, &module, "keyPressedInLastFrame", Box::new(|a,b| keyboard_was_pressed_in_last_frame(a,b)));
        add_function(&guard, &module, "isMouseKeyPressed", Box::new(|a,b| mouse_is_key_pressed(a,b)));
        add_function(&guard, &module, "mousePosition", Box::new(|a,b| mouse_position(a,b)));

    }

}

struct InputAPI;

impl InputAPI {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let namespace = registry.register_namespace("Input2", None);

        registry.register_function("isKeyboardsKeyPressed", Some(&namespace), |args: (Data<KeyboardState>, String)| {
            args.0.is_button_down(KeyCode::from_string(&args.1))
        });

        registry.register_function("keyPressedInLastFrame", Some(&namespace), |args: (Data<KeyboardState>, String)| {
            args.0.button_pressed_in_last_frame(KeyCode::from_string(&args.1))
        });

        registry.register_function("isMouseKeyPressed", Some(&namespace), |args: (Data<MouseState>, usize)| {
            args.0.is_button_down(args.1)
        });

    }
}
