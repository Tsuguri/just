use crate::scene::math::*;
use super::js;
use js::{
    ContextGuard,
    value::{
        Value,
        function::CallbackInfo,
    },
};
use crate::scene::{GameObjectId, WorldData, Hardware};

use super::api_helpers::*;
use crate::scene::scripting::InternalTypes;

struct GameObjectData {
    id: GameObjectId,
}

fn get_position(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let external = args.this.into_external().unwrap();

    let prototypes = prototypes(&ctx);


    let world = world(&ctx);
    let this = unsafe {external.value::<GameObjectData>()};

    let pos = world.get_global_position(this.id);

    let obj = js::value::External::new(guard, Box::new(pos));
    obj.set_prototype(guard, prototypes[&InternalTypes::Vec3].clone());

    println!("{:?}  jest ok", pos);

    Result::Ok(js::value::null(guard))
}


fn test_go_function(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    println!("lol");
    Result::Ok(js::value::null(guard))
}

impl super::JsScriptEngine {
    pub fn create_external_prototype(guard: &ContextGuard) -> js::value::Object {
        let obj =js::value::Object::new(guard);

        let fun = js::value::Function::new(guard, Box::new(|a,b| test_go_function(a,b)));
        obj.set(&guard, js::Property::new(&guard, "test"), fun);

        let fun2 = js::value::Function::new(guard, Box::new(|a,b| get_position(a,b)));
        obj.set(&guard, js::Property::new(&guard, "get_position"), fun2);
        obj
    }

    pub fn create_script_external(&self, guard: &ContextGuard, id: GameObjectId) -> js::value::Value {
        let obj = js::value::External::new(guard, Box::new(GameObjectData { id }));
        obj.set_prototype(guard, (self.prototypes[&InternalTypes::GameObject]).clone()).unwrap();
        obj.into()
    }
    pub fn create_game_object_api(&mut self) {
        let guard = self.guard();
        let ext_prototype = Self::create_external_prototype(&guard);
        drop(guard);

        self.prototypes.insert(InternalTypes::GameObject, ext_prototype);
    }
}