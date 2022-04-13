// use std::collections::{HashMap, HashSet};
// use std::mem::ManuallyDrop;

//use crate::ui::UiEvent;
pub use v8;

use just_core::ecs::prelude::*;
use just_core::traits::scripting::ScriptingEngine;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsEngineConfig {
    pub source_root: String,
}

pub struct V8Engine {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
}

fn println_callback(_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
    println!("hard stuff");
}
impl V8Engine {
    fn create_global_object_template<'a>(
        higher_scope: &mut v8::HandleScope<'a, ()>,
    ) -> v8::Local<'a, v8::ObjectTemplate> {
        let scope = &mut v8::EscapableHandleScope::new(higher_scope);
        let global_template = v8::ObjectTemplate::new(scope);

        // create built in methods?
        let key = v8::String::new(scope, "lol").unwrap();
        let val = v8::FunctionTemplate::new(scope, println_callback);
        // val.set_name(key);
        global_template.set(key.into(), val.into());

        scope.escape(global_template)
    }

    pub fn create(v8_flags: Vec<String>) -> Self {
        if v8_flags.len() > 0 {
            v8::V8::set_flags_from_command_line(v8_flags);
        }
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let create_params = v8::Isolate::create_params();
        let mut isolate = v8::Isolate::new(create_params);
        let context = {
            let scope = &mut v8::HandleScope::new(&mut isolate);
            let global_template = Self::create_global_object_template(scope);
            let context = v8::Context::new_from_template(scope, global_template);
            v8::Global::new(scope, context)
        };

        Self { isolate, context }
    }

    pub fn run(&mut self, source: &str) {
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, self.context.clone());

        let try_catch = &mut v8::TryCatch::new(scope);
        let filepath = v8::String::new(try_catch, "heheszki").unwrap();
        let code = v8::String::new(try_catch, source).unwrap();
        let result = v8::Script::compile(try_catch, code, None)
            .and_then(|script| script.run(try_catch))
            .map_or_else(|| Err(try_catch.stack_trace().unwrap()), Ok);
        let strong = match result {
            Ok(v) => Ok(v.to_string(try_catch).unwrap().to_rust_string_lossy(try_catch)),
            Err(e) => Err(e.to_string(try_catch).unwrap().to_rust_string_lossy(try_catch)),
        };
        println!("{:?}", strong);
    }
    pub fn lol(&mut self) {
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, self.context.clone());
        let test_object = v8::Object::new(scope);
    }
}

impl ScriptingEngine for V8Engine {
    type Config = JsEngineConfig;

    fn create(config: &Self::Config, world: &mut World) -> Self {
        V8Engine::create(vec![])
    }

    fn create_script(&mut self, gameobject_id: Entity, typ: &str, world: &mut World) {}

    fn update(&mut self, world: &mut World) {}
}
// #[macro_use]
// mod api_helpers;

// mod env;
// mod game_object_api;
// mod registry_impl;
//mod resources_api;
//mod world_api;
// use game_object_api::GameObjectApi;

// use env::JsEnvironment;

// //use game_object_api::GameObjectApi;
// //pub use resources_api::MeshData;
// //pub use resources_api::TextureData;

// pub struct EHM(HashMap<std::any::TypeId, js::value::Object>);

// impl Default for EHM {
//     fn default() -> Self {
//         Self(HashMap::new())
//     }
// }

// impl std::ops::Deref for EHM {
//     type Target = HashMap<std::any::TypeId, js::value::Object>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl std::ops::DerefMut for EHM {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
// unsafe impl Send for EHM {}
// unsafe impl Sync for EHM {}

// impl EHM {
//     pub fn get_prototype<T: 'static>(&self) -> &js::value::Object {
//         &self.0[&std::any::TypeId::of::<T>()]
//     }
// }

// pub struct JsScriptEngine {
//     _runtime: js::Runtime,
//     external_types_prototypes: EHM,
//     context: ManuallyDrop<js::Context>,
//     //ui_events_handler: EventDispatcher<UiEvent>,
// }

// pub struct JsScript {
//     js_object: js::value::Object,
//     update: Option<js::value::Function>,
// }

// unsafe impl Send for JsScript {}
// unsafe impl Sync for JsScript {}

// pub struct ScriptCreationData {
//     object: Entity,
//     script_type: String,
// }

// pub struct ScriptCreationQueue {
//     q: Vec<ScriptCreationData>,
// }

// impl JsScript {
//     pub fn new(object: js::value::Object, guard: &js::ContextGuard) -> Self {
//         let update = object
//             .get(guard, js::Property::new(guard, "update"))
//             .into_function();
//         Self {
//             js_object: object,
//             update,
//         }
//     }
// }

// impl std::ops::Drop for JsScriptEngine {
//     fn drop(&mut self) {
//         unsafe {
//             self.external_types_prototypes.clear();
//             ManuallyDrop::drop(&mut self.context);
//         }
//     }
// }

// use std::path::Path;

// struct ScriptFactory {}

// impl ScriptFactory {
//     fn from_code(
//         guard: &js::ContextGuard,
//         _name: String,
//         _path: &Path,
//         code: &str,
//     ) -> Result<js::value::Function, js::Error> {
//         let def = js::script::parse(guard, code)?;
//         let factory = def.construct(&guard, guard.global(), &[])?;
//         let factory = match factory.into_function() {
//             Some(elem) => elem,
//             None => return Result::Err(js::Error::ScriptCompilation("Not a function".to_string())),
//         };
//         Result::Ok(factory)
//     }
//     fn from_path(guard: &js::ContextGuard, path: &Path) -> Result<js::value::Function, js::Error> {
//         let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

//         let code = std::fs::read_to_string(path).unwrap();
//         Self::from_code(guard, name, path, &code)
//     }
// }

// impl JsScriptEngine {
//     pub fn guard(&self) -> js::ContextGuard {
//         self.context.make_current().unwrap()
//     }

//     fn create_api(&mut self) {
//         // Important: GameObjectApi has to be added BEFORE other systems
//         // TODO: move other systems outside of scripting engine and move GO api creation into configure/initialization steps.
//         // TODO: load only needed api?
//         GameObjectApi::register(self);
//         self.create_component_api();
//         //self.create_resources_api();
//     }

//     fn configure(&mut self, config: &JsEngineConfig) {
//         self.create_api();
//         let guard = self.guard();
//         let go = guard.global();

//         Self::load_at_path(&guard, &go, Path::new(&config.source_root)).unwrap();
//     }

//     pub fn run_with<T, F: FnOnce(&js::ContextGuard) -> T>(&self, callback: F) -> T {
//         let p = self.guard();
//         callback(&p)
//     }

//     fn load_at_path(
//         guard: &js::ContextGuard,
//         parent: &js::value::Object,
//         directory: &std::path::Path,
//     ) -> Result<(), &'static str> {
//         println!("loading scripts from: {:?}", directory);

//         let paths = std::fs::read_dir(directory).map_err(|_| "counldn't read directory")?;
//         for path in paths {
//             let path = path.map_err(|_| "error reading script directory")?;

//             if path.path().is_dir() {
//                 let p = path.path();
//                 let p2 = p.file_stem().unwrap();
//                 let namespace = match p2.to_str() {
//                     Option::None => return Result::Err("invalid character in namespace string"),
//                     Option::Some(name) => name,
//                 };
//                 println!("creating namespace: {:?}", namespace);
//                 let obj = js::value::Object::new(guard);
//                 Self::load_at_path(guard, &obj, &path.path())?;
//                 parent.set(guard, js::Property::new(guard, namespace), obj);
//             } else {
//                 let p = path.path();
//                 let p2 = p.file_stem().unwrap().to_str().unwrap();
//                 let factory = ScriptFactory::from_path(guard, &p).unwrap();
//                 parent.set(guard, js::Property::new(guard, p2), factory);
//             }
//         }

//         Result::Ok(())
//     }
// }

// /*
// impl DispatchableEvent for UiEvent {
//     type Hash = (
//         Entity,
//         std::mem::Discriminant<crate::ui::UiEventType>,
//         usize,
//     );
//     fn hash(&self) -> Self::Hash {
//         let d = std::mem::discriminant(&self.event_type);
//         let add = match self.event_type {
//             _ => 0,
//         };
//         (self.entity, d, add)
//     }

//     fn invoke_params(&self, encoder: &ParamEncoder) -> js::value::Value {
//         js::value::null(encoder.guard)
//     }
// }
// use just_input::InputEvent;
// impl DispatchableEvent for InputEvent {
//     type Hash = (std::mem::Discriminant<InputEvent>, usize);
//     fn hash(&self) -> Self::Hash {
//         let d = std::mem::discriminant(self);
//         let add = match self {
//             InputEvent::KeyPressed(x) => x.to_usize(),
//             InputEvent::KeyReleased(x) => x.to_usize(),
//             InputEvent::MouseMoved(..) => 0,
//             InputEvent::MouseButtonPressed(x) => *x,
//             InputEvent::MouseButtonReleased(x) => *x,
//         };
//         (d, add)
//     }

//     fn invoke_params(&self, encoder: &ParamEncoder) -> js::value::Value {
//         match self {
//             InputEvent::MouseMoved(pos) => encoder.encode_v2(*pos),
//             _ => js::value::null(encoder.guard),
//         }
//     }
// }

// */
// impl ScriptingEngine for JsScriptEngine {
//     type Config = JsEngineConfig;

//     fn create(config: &Self::Config, world: &mut World) -> Self {
//         world.resources.insert(ScriptCreationQueue { q: vec![] });
//         let runtime = js::Runtime::new().unwrap();
//         let context = js::Context::new(&runtime).unwrap();
//         //let ui_events_handler = EventDispatcher::<UiEvent>::create(world);
//         let mut engine = Self {
//             _runtime: runtime,
//             context: ManuallyDrop::new(context),
//             //ui_events_handler,
//             external_types_prototypes: Default::default(),
//         };
//         engine.configure(config);
//         engine
//     }

//     fn create_script(&mut self, id: Entity, typ: &str, world: &mut World) {
//         let command = format!("new {}();", typ);
//         let env = JsEnvironment::set_up(
//             &self.context,
//             world,
//             &self.external_types_prototypes,
//         );

//         let guard = self.guard();

//         let obj = js::script::eval(&guard, &command).unwrap();

//         let prot = self.create_go_external(&guard, id);
//         let obj = obj.into_object().unwrap();

//         obj.set(&guard, js::Property::new(&guard, "go"), prot);

//         let script = JsScript::new(obj, &guard);
//         world.add_component(id, script);

//         env.drop(&self.context)
//     }

//     fn update(&mut self, world: &mut World) {
//         let guard = self.context.make_current().unwrap();

//         let env = JsEnvironment::set_up(
//             &self.context,
//             world,
//             &self.external_types_prototypes,
//         );

//         let query = <Read<JsScript>>::query();

//         //self.ui_events_handler.dispatch(&guard, world);

//         for (_entity_id, script) in query.iter_entities_immutable(world) {
//             match &script.update {
//                 None => (),
//                 Some(fun) => {
//                     use failure::Fail;
//                     match fun.call_with_this(&guard, &script.js_object, &[]) {
//                         Ok(..) => (),
//                         Err(err) => {
//                             println!("error: {:?}", err);
//                             match err.backtrace() {
//                                 None => (),
//                                 Some(x) => {
//                                     println!("{:?}", x);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//         env.drop(&self.context);

//         drop(guard);

//         let to_create: Vec<_> = std::mem::replace(
//             &mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q,
//             vec![],
//         );

//         for data in to_create {
//             self.create_script(data.object, &data.script_type, world);
//         }
//     }
// }

// #[derive(Copy, Clone, Debug)]
// pub enum JsRuntimeError {
//     NotEnoughParameters,
//     WrongTypeParameter,
//     ExpectedParameterNotPresent,
//     TypeNotRegistered,
//     ComponentNotPresent,
// }

// struct EventHandler {
//     object: js::value::Object,
//     handler: js::value::Function,
// }

// impl PartialEq for EventHandler {
//     fn eq(&self, other: &Self) -> bool {
//         self.object.as_raw() == other.object.as_raw()
//             && self.handler.as_raw() == other.handler.as_raw()
//     }
// }

// impl Eq for EventHandler {}
// impl std::hash::Hash for EventHandler {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.object.as_raw().hash(state);
//         self.handler.as_raw().hash(state);
//     }
// }

// use js::ContextGuard;

// struct ParamEncoder<'a> {
//     guard: &'a ContextGuard<'a>,
//     prototypes: &'a EHM,
// }

// impl<'a> ParamEncoder<'a> {
//     pub fn encode_float(value: f32, guard: &ContextGuard) -> js::value::Value {
//         js::value::Number::from_double(guard, value as f64).into()
//     }

//     pub fn encode_vec2(
//         value: just_core::math::Vec2,
//         guard: &ContextGuard,
//         prototypes: &EHM,
//     ) -> js::value::Value {
//         let ob = js::value::External::new(guard, Box::new(value));
//         ob.set_prototype(guard, prototypes[&std::any::TypeId::of::<just_core::math::Vec2>()].clone()).unwrap();
//         ob.into()
//     }

//     pub fn encode_f32(&self, value: f32) -> js::value::Value {
//         Self::encode_float(value, self.guard)
//     }

//     pub fn encode_v2(&self, value: just_core::math::Vec2) -> js::value::Value {
//         Self::encode_vec2(value, self.guard, self.prototypes)
//     }
// }
