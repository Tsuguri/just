use super::js;

use js::{
    ContextGuard,
    value::function::CallbackInfo,
    value::Value,
};

use super::api_helpers::*;
use super::game_object_api::GameObjectData;
use crate::scripting::InternalTypes;

fn gameobject_find_by_name(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    debug_assert_eq!(args.arguments.len(), 1);
    let ctx = guard.context();
    let world = world(&ctx);

    let prototypes = prototypes(&ctx);
    let name = args.arguments[0].to_string(guard);

    let objs = world.find_by_name(&name);

    let result = js::value::Array::new(guard, objs.len() as u32);

    let proto = prototypes[&InternalTypes::GameObject].clone();
    for (id, val) in objs.iter().enumerate() {


        let obj = js::value::External::new(guard, Box::new(GameObjectData{id: *val}));
        obj.set_prototype(guard, proto.clone());

        result.set_index(guard, id as u32, obj);
    }

    Result::Ok(result.into())
}

fn gameobject_create(guard: &ContextGuard, args: CallbackInfo)-> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);

    let prototypes = prototypes(&ctx);
    let obj = world.create_gameobject();

    let proto = prototypes[&InternalTypes::GameObject].clone();
    let res = js::value::External::new(guard, Box::new(GameObjectData{id: obj}));
    res.set_prototype(guard, proto);

    Result::Ok(res.into())

}

impl super::JsScriptEngine {
    pub fn create_world_api(&mut self) {
        let module = self.create_api_module("World");
        let guard = self.guard();

        add_function(&guard, &module, "findByName", mf!(gameobject_find_by_name));
        add_function(&guard, &module, "createGameObject", mf!(gameobject_create));

    }
}
