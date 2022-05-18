mod api_helpers;
pub mod engine;
mod game_object_api;
mod param_source;
mod result_encoder;
mod script_creation;
mod script_factory;

use std::collections::HashMap;
// use std::mem::ManuallyDrop;

//use crate::ui::UiEvent;
use game_object_api::GameObjectApi;
use param_source::V8ParametersSource;
use result_encoder::V8ResultEncoder;
pub use v8;

use just_core::{
    ecs::prelude::World,
    traits::scripting::{
        FunctionParameter, FunctionResult, NamespaceId, NativeTypeId, ScriptApiRegistry, TypeCreationError,
    },
};
use script_creation::{ScriptCreationData, ScriptCreationQueue};
use v8::{FunctionCallbackArguments, ReturnValue};

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

pub struct JsScript {
    object: v8::Global<v8::Object>,
    //update: Option<v8::Global<v8::Function>>,
}
unsafe impl Send for JsScript {}
unsafe impl Sync for JsScript {}

pub struct V8ApiRegistry<'a, 'b> {
    scope: &'b mut v8::HandleScope<'a>,
    context: v8::Global<v8::Context>,
    things: HashMap<i32, v8::Local<'a, v8::Object>>,
    last_id: i32,
}

fn creatorFunction<T: 'static>(scope: &mut v8::HandleScope, args: FunctionCallbackArguments, rv: v8::ReturnValue) {}
fn creatorFunction2<P1, R, F1>(scope: &mut v8::HandleScope, args: FunctionCallbackArguments, rv: v8::ReturnValue)
where
    P1: FunctionParameter,
    R: FunctionResult,
    F1: 'static + Fn(P1) -> R,
{
    1 + 2;
}

fn creatorFunc(_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {}

fn create_v8_func<'a, F: for<'c, 'd, 'e> Fn(&'c mut v8::HandleScope, FunctionCallbackArguments, ReturnValue)>(
    scope: &mut v8::HandleScope<'a>,
    cf: F,
) -> v8::Local<'a, v8::Function> {
    let func = v8::Function::builder(&cf).build(scope).unwrap();
    func
}

impl<'a, 'b> V8ApiRegistry<'a, 'b> {
    fn register<F: for<'c, 'd, 'e> Fn(&'c mut v8::HandleScope, FunctionCallbackArguments, ReturnValue)>(
        &mut self,
        cf: F,
    ) {
        let loc_namespace = self.context.open(self.scope).global(self.scope);

        let key = v8::String::new(&mut self.scope, "lolek").unwrap();

        let v2 = v8::Function::builder(creatorFunction::<i32>)
            .build(&mut self.scope)
            .unwrap();
        let v2 = v8::Function::builder(&cf).build(&mut self.scope).unwrap();
        loc_namespace.set(&mut self.scope, key.into(), v2.into());
    }
    fn lolz<F: Fn(i32)>(&mut self, f: F, data: i32) {
        self.register(|a, b, c| {
            f(data);
        });
    }
    fn register_native_type_property<P1, R, F1>(&mut self, getter: F1)
    where
        P1: FunctionParameter,
        R: FunctionResult,
        F1: 'static + Send + Sync + Fn(P1) -> R,
    {
        self.lolz(
            |data| {
                let _ = data + data;
            },
            13,
        );
        //self.register(creatorFunction2::<P1, R, F1>);
        //self.register
    }
}

impl<'a, 'b> ScriptApiRegistry<'a, 'b> for V8ApiRegistry<'a, 'b> {
    fn register_namespace(&mut self, name: &str, parent: Option<NamespaceId>) -> NamespaceId {
        let key = v8::String::new(&mut self.scope, name).unwrap();
        let obj = v8::Object::new(&mut self.scope);
        match parent {
            Some(x) => {
                let par = self.things.get(&x).unwrap();
                par.set(&mut self.scope, key.into(), obj.clone().into());
            }
            None => {
                let context = self.context.open(self.scope);
                context
                    .global(self.scope)
                    .set(&mut self.scope, key.into(), obj.clone().into());
            }
        }

        let id = self.last_id + 1;
        self.last_id += 1;

        self.things.insert(id, obj);
        id
    }

    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<NamespaceId>, fc: F)
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R,
    {
        let loc_namespace = match namespace {
            Some(id) => self.things.get(&id).unwrap().clone(),
            None => self.context.open(self.scope).global(self.scope),
        };
        let key = v8::String::new(&mut self.scope, name).unwrap();
        let v8_func = create_v8_func(&mut self.scope, |a, b, mut c| {
            // read world from context data somehow.
            //let mut world = a.get_slot::<&'static mut World>().unwrap();
            let mut param_source = V8ParametersSource::new(a, &b);

            let result = fc(P::read(&mut param_source).unwrap());
            //
            let mut encoder = V8ResultEncoder::new(a);
            c.set(result.into_script_value(&mut encoder));
        });
        loc_namespace.set(&mut self.scope, key.into(), v8_func.into());
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

    fn register_native_type_method(
        &mut self,
        _type: NativeTypeId,
        name: &str,
        fc: impl v8::MapFnTo<v8::FunctionCallback>,
    ) -> Result<(), TypeCreationError> {
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
