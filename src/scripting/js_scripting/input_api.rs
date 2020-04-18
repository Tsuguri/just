use super::js;

use js::{
    ContextGuard,
    value::function::CallbackInfo,
    value::Value,
};

use super::api_helpers::*;
use crate::input::KeyCode;
use crate::scripting::InternalTypes;
use crate::math::Vec3;


fn keyboard_is_key_pressed(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value>{
    let ctx = guard.context();
    let keyboard = keyboard(&ctx);
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].to_string(guard);

    let result = keyboard.is_button_down(KeyCode::from_string(&arg));
    Result::Ok(js::value::Boolean::new(guard, result).into())
}

fn keyboard_was_pressed_in_last_frame(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let keyboard = keyboard(&ctx);
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].to_string(guard);

    let result = keyboard.button_pressed_in_last_frame(KeyCode::from_string(&arg));
    Result::Ok(js::value::Boolean::new(guard, result).into())

}

fn mouse_is_key_pressed(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let mouse = mouse(&ctx);
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].clone().into_number().unwrap();
    let arg_value = arg.value();

    let result = mouse.is_button_down(arg_value as usize);
    Result::Ok(js::value::Boolean::new(guard, result).into())
}

fn mouse_position(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let mouse = mouse(&ctx);
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