pub struct JsScript;

use crate::scene::traits::{Controller, ScriptingEngine, GameObjectId};

use chakracore as js;
use std::mem::ManuallyDrop;

mod math_api;

pub struct JsScriptEngine {
    _runtime: js::Runtime,
    context: ManuallyDrop<js::Context>,

}

#[cfg(test)]
impl JsScriptEngine {
    pub fn without_scripts() -> Self {
        let runtime = js::Runtime::new().unwrap();
        let context = js::Context::new(&runtime).unwrap();
        let mut engine = Self {
            _runtime: runtime,
            context: ManuallyDrop::new(context),
        };

        engine.create_api();
        engine
    }
}

impl std::ops::Drop for JsScriptEngine {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.context);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsEngineConfig {
    pub source_root: String,
}

impl JsScriptEngine {
    pub fn guard<'a>(&'a self) -> js::ContextGuard<'a> {
        self.context.make_current().unwrap()
    }
    fn create_api(&mut self) {
        self.create_math_api();
    }

    fn configure(&mut self, config: &JsEngineConfig) {}

    pub fn run_with<T, F: FnOnce(&js::ContextGuard)-> T>(&self, callback: F)-> T {
        let p = self.guard();
        callback(&p)
    }
}

impl ScriptingEngine for JsScriptEngine {
    type Controller = JsScript;
    type Config = JsEngineConfig;

    fn create(config: &Self::Config) -> Self {
        let runtime = js::Runtime::new().unwrap();
        let context = js::Context::new(&runtime).unwrap();
        let mut engine = Self {
            _runtime: runtime,
            context: ManuallyDrop::new(context),
        };
        engine.configure(config);
        engine
    }
}

impl Controller for JsScript {
    fn prepare(&mut self) {}

    fn update(&mut self) {}
    fn init(&mut self) {}
    fn destroy(&mut self) {}

    fn get_type_name(&self) -> String { String::new() }

    fn set_bool_property(&mut self, name: &str, value: bool) {}
    fn set_int_property(&mut self, name: &str, value: i64) {}
    fn set_float_property(&mut self, name: &str, value: f32) {}
    fn set_string_property(&mut self, name: &str, value: String) {}

    fn set_controller_property(&mut self, name: &str, value: &Self) {}
    fn set_gameobject_property(&mut self, name: &str, value: GameObjectId) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::math::*;

    #[test]
    fn simple() {
        let engine = JsScriptEngine::without_scripts();
    }

    #[test]
    fn vector_api_no_args() {
        let engine = JsScriptEngine::without_scripts();
        let ret = engine.run_with(|x| {
            js::script::eval(x, "
            new Math.Vector()
            ").unwrap()
        });
        assert!(ret.is_external());
        unsafe {
            let obj = ret.into_external().unwrap();
            let p = obj.value::<Vec3>();
            assert!(p.data[0] == 0.0f32);
            assert!(p.data[1] == 0.0f32);
            assert!(p.data[2] == 0.0f32);
        }
    }

    #[test]
    fn vector_3args() {
        let engine = JsScriptEngine::without_scripts();
        let ret = engine.run_with(|x| {
            js::script::eval(x, "new Math.Vector(12, 3.0, 4.5)")
        });
        assert!(ret.is_ok());
        let ret = ret.unwrap();
        assert!(ret.is_external());
        unsafe {
            let obj = ret.into_external().unwrap();
            let p = obj.value::<Vec3>();
            assert!(p.data[0]== 12.0f32);
            assert!(p.data[1]== 3.0f32);
            assert!(p.data[2]== 4.5f32);
        }

    }
    #[test]
    fn vector_bad_args() {
        let engine = JsScriptEngine::without_scripts();
        let ret = engine.run_with(|x| {
            let r1 = js::script::eval(x, "new Math.Vector(1.0)");
            let r2 = js::script::eval(x, "new Math.Vector(1.0, 2.0)");
            let r3 = js::script::eval(x, "new Math.Vector(\"wow\", \"wow\", \"wut\")");
            let r4 = js::script::eval(x, "new Math.Vector(1.0, 2.0, 3.0, 4.0)");
            (r1,r2,r3,r4)
        });
        assert!(!ret.0.is_ok());
        assert!(!ret.1.is_ok());
        assert!(!ret.2.is_ok());
        assert!(!ret.3.is_ok());
    }
}
