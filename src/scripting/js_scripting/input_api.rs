use super::js;

use js::{
    ContextGuard,
    value::function::CallbackInfo,
    value::Value,
};

use super::api_helpers::*;
use crate::input::KeyCode;


fn keyboard_is_key_pressed(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value>{
    let ctx = guard.context();
    let keyboard = keyboard(&ctx);
    debug_assert_eq!(1, args.arguments.len());
    let arg = args.arguments[0].to_string(guard);

    let result = keyboard.is_button_down(KeyCode::from_string(&arg));
    Result::Ok(js::value::Boolean::new(guard, result).into())
}

impl super::JsScriptEngine {
    pub fn create_input_api(&mut self) {
        let module = self.create_api_module("Input");
        let guard = self.guard();

        add_function(&guard, &module, "isKeyPressed", Box::new(|a,b| keyboard_is_key_pressed(a,b)));

    }

}