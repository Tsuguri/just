pub struct MockScript;
pub struct MockScriptEngine;
use crate::scene::traits::{Controller, ScriptingEngine, GameObjectId};
use crate::scene::MockEngine;

impl MockEngine{
    pub fn mock()->Self{
        MockEngine::new(&MockEngineConfig(1), &1i32, &1i32)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct MockEngineConfig(i32);

impl From<i32> for MockEngineConfig{
    fn from(value:i32)-> Self{
        MockEngineConfig(value)
    }
}

impl ScriptingEngine for MockScriptEngine {
    type Controller = MockScript;
    type Config = MockEngineConfig;

    fn create(_config: &Self::Config) -> Self{
        Self{}
    }

}
impl Controller for MockScript {
    fn prepare(&mut self) {

    }

    fn update(&mut self) {

    }
    fn init(&mut self){}
    fn destroy(&mut self){}

    fn get_type_name(&self) -> String{String::new()}

    fn set_bool_property(&mut self, _name: &str, _value: bool){}
    fn set_int_property(&mut self, _name: &str, _value: i64){}
    fn set_float_property(&mut self, _name: &str, _value: f32){}
    fn set_string_property(&mut self, _name: &str, _value: String){}

    fn set_controller_property(&mut self, _name: &str, _value: &Self){}
    fn set_gameobject_property(&mut self, _name: &str, _value: GameObjectId){}
}
