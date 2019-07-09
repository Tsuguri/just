use super::GameObjectId;
pub mod js_scripting;
pub use js_scripting::*;

#[cfg(test)]
pub mod test_scripting;

use serde::Deserialize;

pub trait Controller {
    fn prepare(&mut self);
    fn update(&mut self);
    fn init(&mut self);
    fn destroy(&mut self);

    fn get_type_name(&self) -> String;

    fn set_bool_property(&mut self, name: &str, value: bool);
    fn set_int_property(&mut self, name: &str, value: i64);
    fn set_float_property(&mut self, name: &str, value: f32);
    fn set_string_property(&mut self, name: &str, value: String);

    fn set_controller_property(&mut self, name: &str, value: &Self);
    fn set_gameobject_property(&mut self, name: &str, value: GameObjectId);
}


pub trait ScriptingEngine {
    type Controller: Controller;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) ->Self;
}

