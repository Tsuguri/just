use super::js;
use js::{
    value::{function::CallbackInfo, Value},
    ContextGuard,
};

use super::api_helpers::*;
use super::{ScriptCreationData, ScriptCreationQueue};

use just_core::ecs::prelude::Entity;
use just_core::GameObjectData;

use just_core::traits::scripting::{
    ScriptApiRegistry,
    FunctionResult,
    FunctionParameter,
    function_params::*,
};

#[derive(Debug, Copy, Clone)]
pub struct ComponentData<T> {
    pub id: Entity,
    _phantom: std::marker::PhantomData<T>,
}


fn set_script(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let creation_data = &mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q;

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let m = args.arguments[0].clone().into_string().unwrap();

    creation_data.push(ScriptCreationData {
        object: this.id,
        script_type: m.value(),
    });
    Result::Ok(js::value::null(guard))
}

fn get_script(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let script = world.get_component::<super::JsScript>(this.id);
    match script {
        None => Result::Err(js::value::null(guard)),
        Some(x) => Result::Ok(x.js_object.clone().into()),
    }
}

pub struct GameObjectApi {}

impl GameObjectApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let go_type = registry
            .register_native_type("GameObject", None, |arg: GameObjectData| arg)
            .unwrap();
    }
}

impl super::JsScriptEngine {
    pub fn create_go_external(&self, guard: &ContextGuard, id: Entity) -> js::value::Value {
        let obj = js::value::External::new(guard, Box::new(GameObjectData { id }));
        let type_id = std::any::TypeId::of::<GameObjectData>();
        obj.set_prototype(guard, (self.external_types_prototypes[&type_id]).clone())
            .unwrap();

        obj.into()
    }
}
