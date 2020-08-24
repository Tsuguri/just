use super::js;
use crate::scripting::{EHM, HM};
use crate::traits::ResourceProvider;

use super::ScriptCreationData;
use crate::input::{KeyboardState, MouseState};
use js::value::function::FunctionCallback;
use js::ContextGuard;
use legion::prelude::World;

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

macro_rules! double {
    ($g:ident, $x:expr) => {
        match $x.clone().into_number() {
            None => return Result::Err(js::value::null($g)),
            Some(x) => x.value_double(),
        }
    };
}

macro_rules! make_double {
    ($g:ident, $x:expr) => {
        js::value::Number::from_double($g, $x)
    };
}

macro_rules! setter {
    ($g: ident, $p:ident, $f:ident) => {
        let fun = js::value::Function::new($g, mf!($f));
        $p.set($g, js::Property::new($g, "set"), fun);
    };
}

macro_rules! getter {
    ($g: ident, $p:ident, $f:ident) => {
        let fun = js::value::Function::new($g, mf!($f));
        $p.set($g, js::Property::new($g, "get"), fun);
    };
}

macro_rules! full_prop {
    ($g: ident, $p:ident, $name:expr,$getter:ident, $setter:ident) => {
        let prop = js::value::Object::new($g);
        setter!($g, prop, $setter);
        getter!($g, prop, $getter);
        $p.define_property($g, js::Property::new($g, $name), prop);
    };
}