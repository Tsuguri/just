use crate::traits::{Controller, ScriptingEngine, ResourceManager, ResourceProvider};

use chakracore as js;
use std::mem::ManuallyDrop;
use std::collections::HashMap;


#[macro_use]
mod api_helpers;

mod input_api;
mod math_api;
mod console_api;
mod game_object_api;
mod time_api;
mod world_api;
mod resources_api;
mod der;

use legion::prelude::*;


#[derive(PartialEq, Eq, Hash)]
pub enum InternalTypes {
    GameObject,
    Vec3,
    Quat,
    Matrix,
}

pub struct HM(HashMap<InternalTypes, js::value::Object>);

impl Default for HM {
    fn default() -> Self{
        Self(HashMap::new())
    }
}

impl std::ops::Deref for HM {
    type Target = HashMap<InternalTypes, js::value::Object>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for HM {

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl Send for HM{}
unsafe impl Sync for HM{}

pub struct JsScriptEngine {
    _runtime: js::Runtime,
    context: ManuallyDrop<js::Context>,
    prototypes: HM,
}

pub struct JsScript {
    js_object: js::value::Object,
    update: Option<js::value::Function>,
}

unsafe impl Send for JsScript {}
unsafe impl Sync for JsScript {}

pub struct ScriptCreationData {
    object: Entity,
    script_type: String,
}

pub struct ScriptCreationQueue {
    q: Vec<ScriptCreationData>,
}

impl JsScript {
    pub fn new(object: js::value::Object, guard: &js::ContextGuard) -> Self {
        let update = object.get(guard, js::Property::new(guard, "update")).into_function();
        Self {
            js_object: object,
            update,
        }
    }
}

#[cfg(test)]
impl JsScriptEngine {
    pub fn without_scripts() -> Self {
        let runtime = js::Runtime::new().unwrap();
        let context = js::Context::new(&runtime).unwrap();
        let mut engine = Self {
            _runtime: runtime,
            context: ManuallyDrop::new(context),
            prototypes: Default::default(),
        };

        engine.create_api();
        engine
    }
}

impl std::ops::Drop for JsScriptEngine {
    fn drop(&mut self) {
        unsafe {
            self.prototypes.clear();
            ManuallyDrop::drop(&mut self.context);
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsEngineConfig {
    pub source_root: String,
}

use std::path::Path;


struct ScriptFactory {}

impl ScriptFactory {
    fn from_code(guard: &js::ContextGuard, _name: String, _path: &Path, code: &str) -> Result<js::value::Function, js::Error> {
        let def = js::script::parse(guard, code)?;
        let factory = def.construct(&guard, guard.global(), &[])?;
        let factory = match factory.into_function() {
            Some(elem) => elem,
            None => return Result::Err(js::Error::ScriptCompilation("Not a function".to_string()))
        };
        Result::Ok(factory)
    }
    fn from_path(guard: &js::ContextGuard, path: &Path) -> Result<js::value::Function, js::Error> {
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        let code = std::fs::read_to_string(path).unwrap();
        Self::from_code(guard, name, path, &code)
    }
}

impl JsScriptEngine {
    pub fn guard(&self) -> js::ContextGuard {
        self.context.make_current().unwrap()
    }
    fn create_api(&mut self) {
        self.create_math_api();
        self.create_console_api();
        self.create_game_object_api();
        self.create_time_api();
        self.create_input_api();
        self.create_world_api();
        self.create_resources_api();
    }

    fn configure(&mut self, config: &JsEngineConfig) {
        self.create_api();
        let guard = self.guard();

        //let guard = self.context.make_current().map_err(|err| "couldn't make context current").unwrap();
        let go = guard.global();


        Self::load_at_path(&guard, &go, Path::new(&config.source_root)).unwrap();
    }

    pub fn run_with<T, F: FnOnce(&js::ContextGuard) -> T>(&self, callback: F) -> T {
        let p = self.guard();
        callback(&p)
    }
    fn load_at_path(guard: &js::ContextGuard, parent: &js::value::Object, directory: &std::path::Path) -> Result<(), &'static str> {
        println!("loading scripts from: {:?}", directory);

        let paths = std::fs::read_dir(directory).map_err(|_| "counldn't read directory")?;
        for path in paths {
            let path = path.map_err(|_| "error reading script directory")?;

            if path.path().is_dir() {
                let p = path.path();
                let p2 = p.file_stem().unwrap();
                let namespace = match p2.to_str() {
                    Option::None =>
                        return Result::Err("invalid character in namespace string"),
                    Option::Some(name) => name,
                };
                println!("creating namespace: {:?}", namespace);
                let obj = js::value::Object::new(guard);
                Self::load_at_path(guard, &obj, &path.path())?;
                parent.set(guard, js::Property::new(guard, namespace), obj);
            } else {
                let p = path.path();
                let p2 = p.file_stem().unwrap().to_str().unwrap();
                let factory = ScriptFactory::from_path(guard, &p).unwrap();
                parent.set(guard, js::Property::new(guard, p2), factory);
            }
        }

        Result::Ok(())
    }
}


impl ScriptingEngine for JsScriptEngine {
    type Controller = JsScript;
    type Config = JsEngineConfig;

    fn create(config: &Self::Config, world: &mut World) -> Self {
        world.resources.insert(ScriptCreationQueue{q:vec![]});
        let runtime = js::Runtime::new().unwrap();
        let context = js::Context::new(&runtime).unwrap();
        let mut engine = Self {
            _runtime: runtime,
            context: ManuallyDrop::new(context),
            prototypes: Default::default(),
        };
        engine.configure(config);
        engine
    }

    fn create_script(&mut self, id: Entity, typ: &str, world: &mut World) {
        let command = format!("new {}();", typ);
        let guard = self.guard();

        let obj = js::script::eval(&guard, &command).unwrap();

        let prot = self.create_script_external(&guard, id);
        let obj = obj.into_object().unwrap();

        obj.set(&guard, js::Property::new(&guard, "go"), prot);

        let script = JsScript::new(obj, &guard);
        world.add_component(id, script);

    }

    fn update(&mut self,
              world: &mut World,
              keyboard: &crate::input::KeyboardState,
              mouse: &crate::input::MouseState,
              current_time: f64,

    ) {

        self.set_time(current_time);
        let testing = 33i32;
        //set context data

        let reference = unsafe{
            std::mem::transmute::<&mut World, &'static mut World>(world)
        };
        self.context.insert_user_data::<&mut World>(reference);

        insert(&self.context, keyboard);
        insert(&self.context, mouse);
        insert(&self.context, &self.prototypes);


        let guard = self.guard();

        let query = <(Read<JsScript>)>::query();

        for (_entity_id, script) in query.iter_entities_immutable(world) {
            match &script.update {
                None => (),
                Some(fun) => { fun.call_with_this(&guard, &script.js_object, &[]).unwrap(); }
            }

        }

        drop(guard);

        let to_create : Vec<_> = std::mem::replace(&mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q, vec![]);

        for data in to_create {
            self.create_script(data.object, &data.script_type, world);
        }
        debug_assert!(self.context.remove_user_data::<&mut World>().is_some());

        debug_assert!(self.context.remove_user_data::<&crate::input::KeyboardState>().is_some());
        debug_assert!(self.context.remove_user_data::<&crate::input::MouseState>().is_some());
        debug_assert!(self.context.remove_user_data::<&HM>().is_some());
    }
}

fn insert<T: Send + Sync + 'static>(context: &js::Context, val: &T){
    let reference = unsafe{
        std::mem::transmute::<&T, &'static T>(val)
    };
    context.insert_user_data::<&T>(reference);
}

fn insert_mut<T: Send + Sync + 'static>(context: &js::Context, val: &mut T){
        let reference = unsafe{
            std::mem::transmute::<&mut T, &'static mut T>(val)
        };
        context.insert_user_data::<&mut T>(reference);
    }

impl Controller for JsScript {
    fn prepare(&mut self) {}

    fn init(&mut self) {}
    fn destroy(&mut self) {}

    fn get_type_name(&self) -> String { String::new() }

    fn set_bool_property(&mut self, _name: &str, _value: bool) {}
    fn set_int_property(&mut self, _name: &str, _value: i64) {}
    fn set_float_property(&mut self, _name: &str, _value: f32) {}
    fn set_string_property(&mut self, _name: &str, _value: String) {}

    fn set_controller_property(&mut self, _name: &str, _value: &Self) {}
    fn set_gameobject_property(&mut self, _name: &str, _value: Entity) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::*;

    #[test]
    fn simple() {
        let _engine = JsScriptEngine::without_scripts();
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
            assert!(p.data[0] == 12.0f32);
            assert!(p.data[1] == 3.0f32);
            assert!(p.data[2] == 4.5f32);
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
            (r1, r2, r3, r4)
        });
        assert!(!ret.0.is_ok());
        assert!(!ret.1.is_ok());
        assert!(!ret.2.is_ok());
        assert!(!ret.3.is_ok());
    }
}
