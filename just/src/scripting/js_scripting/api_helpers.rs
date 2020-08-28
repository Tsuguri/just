use super::js;
use crate::scripting::{EHM};

use js::value::function::FunctionCallback;
use js::ContextGuard;
use just_core::ecs::prelude::World;

impl super::JsScriptEngine {
    pub fn create_api_module(&mut self, name: &str) -> js::value::Object {
        let guard = self.guard();
        let global = guard.global();
        let module = js::value::Object::new(&guard);
        global.set(&guard, js::Property::new(&guard, name), module.clone());
        module
    }
}

pub fn add_function(
    guard: &ContextGuard,
    obj: &js::value::Object,
    name: &str,
    fun: Box<FunctionCallback>,
) {
    let fun = js::value::Function::new(guard, fun);
    obj.set(&guard, js::Property::new(&guard, name), fun);
}

pub fn world(ctx: &js::Context) -> &mut World {
    *ctx.get_user_data_mut::<&mut World>().unwrap()
}

pub fn external_prototypes(ctx: &js::Context) -> &EHM {
    *ctx.get_user_data::<&EHM>().unwrap()
}

macro_rules! mf {
    ($i: ident) => {
        Box::new(|a, b| $i(a, b))
    };
}
