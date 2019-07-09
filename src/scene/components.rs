use super::*;

impl<E: ScriptingEngine> Scene<E> {
    pub fn get_script(&mut self) -> Option<&mut scripting::JsScript> {
        Option::None

    }

}