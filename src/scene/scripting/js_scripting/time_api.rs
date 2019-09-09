use crate::scene::math::*;
use super::js;
use js::ContextGuard;
use js::value::Value;

use super::api_helpers::*;
use crate::scene::scripting::InternalTypes;
use chakracore::value::function::CallbackInfo;
use super::api_helpers::*;


impl super::JsScriptEngine {
    pub fn create_time_api(&mut self) {
        let module = self.create_api_module("Time");
        let guard = self.guard();

        let val = js::value::Number::from_double(&guard, 0.0f64);

        //module.set(&guard, js::Property::new(&guard, "dt"), val.into());
        module.set(&guard, js::Property::new(&guard, "elapsed"), val);

    }

    pub fn set_time(&mut self, current_time: f64) {
        let guard = self.guard();
        let global = guard.global();

        let mut module = global.get(&guard, js::Property::new(&guard, "Time")).into_object().unwrap();

        let val = js::value::Number::from_double(&guard, current_time);

        //module.set(&guard, js::Property::new(&guard, "dt"), val.into());
        module.set(&guard, js::Property::new(&guard, "elapsed"), val);

    }
}
