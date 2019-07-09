pub struct JsScript;
use super::{Controller, ScriptingEngine, GameObjectId};

use chakracore as js;
use std::mem::ManuallyDrop;

pub struct JsEngine{
    runtime: js::Runtime,
    context: ManuallyDrop<js::Context>,

}

impl std::ops::Drop for JsEngine {
    fn drop(&mut self) {
        unsafe{
            ManuallyDrop::drop(&mut self.context);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsEngineConfig {
    source_root: String,
}

impl ScriptingEngine for JsEngine {
    type Controller = JsScript;
    type Config = i32;

    fn create(config: &Self::Config) -> Self{
        let runtime = js::Runtime::new().unwrap();
        let context = js::Context::new(&runtime).unwrap();
        JsEngine{
            runtime,
            context: ManuallyDrop::new(context)
        }
    }

}
impl Controller for JsScript {
    fn prepare(&mut self) {

    }

    fn update(&mut self) {

    }
    fn init(&mut self){}
    fn destroy(&mut self){}

    fn get_type_name(&self) -> String{String::new()}

    fn set_bool_property(&mut self, name: &str, value: bool){}
    fn set_int_property(&mut self, name: &str, value: i64){}
    fn set_float_property(&mut self, name: &str, value: f32){}
    fn set_string_property(&mut self, name: &str, value: String){}

    fn set_controller_property(&mut self, name: &str, value: &Self){}
    fn set_gameobject_property(&mut self, name: &str, value: GameObjectId){}
}
