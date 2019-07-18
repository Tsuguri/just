use super::*;

impl<E: ScriptingEngine, HW: Hardware> Engine<E, HW> {
    pub fn get_script(&mut self) -> Option<&mut scripting::JsScript> {
        Option::None
    }
}