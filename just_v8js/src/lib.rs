use std::collections::{HashMap, HashSet};
// use std::mem::ManuallyDrop;

//use crate::ui::UiEvent;
use core::convert::{TryFrom, TryInto};
use std::ffi::c_void;
pub use v8;

use just_core::ecs::prelude::*;
use just_core::traits::scripting::{
    FunctionParameter, FunctionResult, NamespaceId, NativeTypeId, RuntimeError, ScriptApiRegistry, TypeCreationError,
};
use v8::{Handle, HandleScope, Script, V8};

pub struct EHM(HashMap<std::any::TypeId, v8::Object>);

impl Default for EHM {
    fn default() -> Self {
        Self(HashMap::new())
    }
}

impl std::ops::Deref for EHM {
    type Target = HashMap<std::any::TypeId, v8::Object>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for EHM {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl Send for EHM {}
unsafe impl Sync for EHM {}

impl EHM {
    pub fn get_prototype<T: 'static>(&self) -> &v8::Object {
        &self.0[&std::any::TypeId::of::<T>()]
    }
}

pub struct ScriptCreationData {
    object: Entity,
    script_type: String,
}

pub struct ScriptCreationQueue {
    q: Vec<ScriptCreationData>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsEngineConfig {
    pub source_root: String,
    pub v8_args: Vec<String>,
}

pub struct V8Engine {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    prototypes: EHM,
    controllers_constructors: HashMap<String, v8::Global<v8::Function>>,
}

pub struct JsScript {
    object: v8::Global<v8::Object>,
    //update: Option<v8::Global<v8::Function>>,
}
unsafe impl Send for JsScript {}
unsafe impl Sync for JsScript {}

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

    pub fn create_without_api(args: JsEngineConfig) -> Self {
        Self::create_with_api(args, |_| {})
    }

    fn fill<'a>(scope: HandleScope<'a>, context: v8::Global<v8::Context>) {
        //let ctx = context.open(scope);
        //let global = ctx.global(scope);
    }

    pub fn create_with_api<Builder>(args: JsEngineConfig, builder: Builder) -> Self
    where
        Builder: for<'a, 'b, 'c> FnOnce(&'c mut V8ApiRegistry<'a, 'b>),
    {
        if !args.v8_args.is_empty() {
            v8::V8::set_flags_from_command_line(args.v8_args);
        }
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let create_params = v8::Isolate::create_params();
        let mut isolate = v8::Isolate::new(create_params);
        let context = {
            let mut scope = v8::HandleScope::new(&mut isolate);
            let global_template = Self::create_global_object_template(&mut scope);
            //fill_api(&mut registry);
            let context = v8::Context::new_from_template(&mut scope, global_template);
            v8::Global::new(&mut scope, context)
        };
        let mut controllers_constructors = HashMap::new();
        {
            let scope = &mut v8::HandleScope::with_context(&mut isolate, context.clone());
            let cont = context.open(scope);
            Self::load_at_path(
                &mut controllers_constructors,
                scope,
                &"user",
                Path::new(&args.source_root),
            )
            .unwrap();
        }
        {
            let mut scope = v8::HandleScope::with_context(&mut isolate, context.clone());
            let mut sar = V8ApiRegistry {
                scope: &mut scope,
                things: Default::default(),
                last_id: 0,
            };
            builder(&mut sar);
        }

        for item in controllers_constructors.iter() {
            println!("   loaded controller: {}", item.0);
        }

        Self {
            isolate,
            context,
            prototypes: Default::default(),
            controllers_constructors,
        }
    }

    pub fn run(&mut self, source: &str) {
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, self.context.clone());

        let try_catch = &mut v8::TryCatch::new(scope);
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
    fn load_at_path(
        constructors: &mut HashMap<String, v8::Global<v8::Function>>,
        scope: &mut v8::HandleScope,
        namespace: &str,
        directory: &std::path::Path,
    ) -> Result<(), &'static str> {
        println!("loading scripts from: {:?} into {}", directory, namespace);

        let paths = std::fs::read_dir(directory).map_err(|_| "counldn't read directory")?;
        for path in paths {
            let path = path.map_err(|_| "error reading script directory")?;

            if path.path().is_dir() {
                let p = path.path();
                let p2 = p.file_stem().unwrap();
                let namespace_suffix = match p2.to_str() {
                    Option::None => return Result::Err("invalid character in namespace string"),
                    Option::Some(name) => name,
                };
                println!("creating namespace: {:?}", namespace_suffix);
                Self::load_at_path(
                    constructors,
                    scope,
                    &format!("{}.{}", namespace, namespace_suffix),
                    &path.path(),
                )?;
            } else {
                let p = path.path();
                let p2 = p.file_stem().unwrap().to_str().unwrap();
                let controller_name = format!("{}.{}", namespace, p2);
                println!("loading controller: {}", controller_name);
                let obj = ScriptFactory::from_path(scope, &p).unwrap();
                constructors.insert(controller_name, v8::Global::new(scope, obj));
            }
        }

        Result::Ok(())
    }
}

use std::path::Path;

struct ScriptFactory {}

impl ScriptFactory {
    fn from_code<'a>(
        scope: &mut v8::HandleScope<'a>,
        name: String,
        _path: &Path,
        code: &str,
    ) -> Result<v8::Local<'a, v8::Function>, ()> {
        // let def = js::script::parse(guard, code)?;
        // let factory = def.construct(&guard, guard.global(), &[])?;
        // let factory = match factory.into_function() {
        //     Some(elem) => elem,
        //     None => return Result::Err(js::Error::ScriptCompilation("Not a function".to_string())),
        // };

        let try_catch = &mut v8::TryCatch::new(scope);
        let code = v8::String::new(try_catch, code).unwrap();
        let result = v8::Script::compile(try_catch, code, None)
            .and_then(|script| script.run(try_catch))
            .map_or_else(|| Err(try_catch.stack_trace().unwrap()), Ok);

        println!("Looking for class {}", name);

        match result {
            Ok(v) => {
                let context = try_catch.get_current_context();
                let global = context.global(try_catch);
                let expected_name = v8::String::new(try_catch, &name).unwrap();
                println!(
                    "context has: {}, {:?}",
                    name,
                    global.has(try_catch, expected_name.into())
                );
                let value = global.get(try_catch, expected_name.into());
                if !value.is_some() {
                    //no expected name
                    return Err(());
                }
                let value = value.unwrap();
                println!("Odpaliło się");
                println!("undefined: {:#?}", value.is_undefined());
                println!("null: {:#?}", value.is_null());
                println!("function: {}", value.is_function());
                if value.is_function() {
                    Ok(v.try_into().unwrap())
                } else {
                    println!("is not a function");
                    Err(())
                }
            }
            Err(_) => Err(()),
        }
        // Result::Ok(factory)
    }
    fn from_path<'a>(scope: &mut v8::HandleScope<'a>, path: &Path) -> Result<v8::Local<'a, v8::Function>, ()> {
        println!("loading code: {}", path.display());
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        let code = std::fs::read_to_string(path).unwrap();
        Self::from_code(scope, name, path, &code)
    }
}

pub struct V8ApiRegistry<'a, 'b> {
    scope: &'b mut v8::HandleScope<'a>,
    things: HashMap<i32, v8::Local<'a, v8::Object>>,
    last_id: i32,
}

impl V8Engine {
    pub fn create<Builder>(config: JsEngineConfig, world: &mut World, builder: Builder) -> Self
    where
        Builder: for<'a, 'b, 'c> FnOnce(&'c mut V8ApiRegistry<'a, 'b>),
    {
        world.resources.insert(ScriptCreationQueue { q: vec![] });
        V8Engine::create_with_api(config, |sar| builder(sar))
    }

    pub fn create_script(&mut self, gameobject_id: Entity, typ: &str, world: &mut World) {
        let script_name = format!("user.{}", typ);
        println!("Attempting to create controllr: {}", script_name);

        if self.controllers_constructors.contains_key(&script_name) {
            println!("  Found requested controller");
            let scope = &mut v8::HandleScope::with_context(&mut self.isolate, self.context.clone());
            let constructor = self.controllers_constructors.get(&script_name).unwrap();
            //let recv = v8::null(scope);
            let controller = constructor.open(scope).new_instance(scope, &vec![]);

            match controller {
                Some(x) => {
                    if x.is_object() {
                        println!("It's an object!!");
                    } else {
                        println!("  Ugh");
                    }

                    let glob_constructor = v8::Global::new(scope, x);
                    let comp = JsScript {
                        object: glob_constructor,
                    };
                    world.add_component(gameobject_id, comp);
                }
                None => {
                    println!("Failed to create {}", script_name);
                }
            }
        } else {
            println!("  Missing controller: {}", script_name);
        }
    }

    pub fn update(&mut self, world: &mut World) {
        // update scripts

        let mut scope = v8::HandleScope::with_context(&mut self.isolate, self.context.clone());
        let query = <Read<JsScript>>::query();
        for (id, script) in query.iter_entities_immutable(world) {
            let controller = script.object.open(&mut scope);
            let handl = script.object.clone();
            let key = v8::String::new(&mut scope, "update").unwrap();

            let update = controller.get(&mut scope, key.into());
            match update {
                Some(x) => {
                    if x.is_function() {
                        let try_catch = &mut v8::TryCatch::new(&mut scope);
                        let func: v8::Local<v8::Function> = x.try_into().unwrap();
                        let script_handle = v8::Local::new(try_catch, handl);
                        func.call(try_catch, script_handle.into(), &vec![]);
                    } else {
                        println!("update is not function");
                    }
                }
                None => {
                    println!("no update here");
                }
            }
        }

        drop(query);
        drop(scope);

        // creating scripts requested in this frame
        let to_create: Vec<_> = std::mem::take(&mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q);

        for data in to_create {
            self.create_script(data.object, &data.script_type, world);
        }
    }
}

impl<'a, 'b> ScriptApiRegistry<'a, 'b> for V8ApiRegistry<'a, 'b> {
    fn register_namespace(&mut self, name: &str, parent: Option<NamespaceId>) -> NamespaceId {
        let key = v8::String::new(&mut self.scope, name).unwrap();
        let obj = v8::Object::new(&mut self.scope);
        match parent {
            Some(x) => {
                let par = self.things.get(&x).unwrap();
                par.set(&mut self.scope, key.into(), obj.into());
            }
            None => {
                //self.global.set(&mut self.scope, key.into(), obj.into());
            }
        }

        let id = self.last_id + 1;
        self.last_id += 1;

        id
    }

    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<NamespaceId>, fc: F)
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R,
    {
    }

    fn register_native_type<T, P, F>(
        &mut self,
        name: &str,
        namespace: Option<NamespaceId>,
        constructor: F,
    ) -> Result<NativeTypeId, TypeCreationError>
    where
        T: 'static,
        P: FunctionParameter,
        F: 'static + Send + Sync + Fn(P) -> T,
    {
        Ok(0)
    }

    fn get_native_type<T: 'static>(&mut self) -> Option<NativeTypeId> {
        None
    }

    fn register_component<T, F>(
        &mut self,
        name: &str,
        namespace: Option<NamespaceId>,
        constructor: F,
    ) -> Result<NativeTypeId, TypeCreationError>
    where
        T: 'static + Send + Sync,
        F: 'static + Send + Sync + Fn() -> T,
    {
        Ok(0)
    }

    fn register_native_type_method<P, R, F>(
        &mut self,
        _type: NativeTypeId,
        name: &str,
        method: F,
    ) -> Result<(), TypeCreationError>
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R,
    {
        Ok(())
    }

    fn register_native_type_property<P1, P2, R, F1, F2>(
        &mut self,
        _type: NativeTypeId,
        name: &str,
        getter: Option<F1>,
        setter: Option<F2>,
    ) where
        P1: FunctionParameter,
        P2: FunctionParameter,
        R: FunctionResult,
        F1: 'static + Send + Sync + Fn(P1) -> R,
        F2: 'static + Send + Sync + Fn(P2),
    {
    }

    fn register_static_property<P1, P2, R, F1, F2>(
        &mut self,
        name: &str,
        namespace: Option<NamespaceId>,
        getter: Option<F1>,
        setter: Option<F2>,
    ) where
        P1: FunctionParameter,
        P2: FunctionParameter,
        R: FunctionResult,
        F1: 'static + Send + Sync + Fn(P1) -> R,
        F2: 'static + Send + Sync + Fn(P2),
    {
    }
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

// pub struct JsScriptEngine {
//     _runtime: js::Runtime,
//     external_types_prototypes: EHM,
//     context: ManuallyDrop<js::Context>,
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
