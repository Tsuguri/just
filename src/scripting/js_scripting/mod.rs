use crate::traits::{
    Controller,
    ScriptingEngine,
    ResourceManager,
    ResourceProvider,
    ScriptApiRegistry,
    FunctionResult,
    FunctionParameter,
};

use chakracore as js;
use std::mem::ManuallyDrop;
use std::collections::{HashMap, HashSet};

use crate::ui::UiEvent;


use super::console_api::ConsoleApi;

#[macro_use]
mod api_helpers;

mod input_api;
mod math_api;
mod game_object_api;
mod time_api;
mod world_api;
mod resources_api;
mod der;

use legion::prelude::*;


#[derive(PartialEq, Eq, Hash)]
pub enum InternalTypes {
    GameObject,
    Vec2,
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
    ui_events_handler: EventDispatcher<UiEvent>,
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
        ConsoleApi::register(self);
        self.create_game_object_api();
        //self.create_time_api();
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

impl DispatchableEvent for UiEvent {
    type Hash = (Entity, std::mem::Discriminant<crate::ui::UiEventType>, usize);
    fn hash(&self) -> Self::Hash {
        let d = std::mem::discriminant(&self.event_type);
        let add = match self.event_type {
            _ => 0,
        };
        (self.entity, d, add)
    }

    fn invoke_params(&self, encoder: &ParamEncoder) -> js::value::Value {
        js::value::null(encoder.guard)
        
    }
}
use crate::input::InputEvent;
impl DispatchableEvent for InputEvent {
    type Hash = (std::mem::Discriminant<InputEvent>, usize);
    fn hash(&self) -> Self::Hash {
        let d = std::mem::discriminant(self);
        let add = match self {
            InputEvent::KeyPressed(x) => x.to_usize(),
            InputEvent::KeyReleased(x) => x.to_usize(),
            InputEvent::MouseMoved(..) => 0,
            InputEvent::MouseButtonPressed(x) => *x,
            InputEvent::MouseButtonReleased(x) => *x,
        };
        (d, add)
    }

    fn invoke_params(&self, encoder: &ParamEncoder) -> js::value::Value {
        match self {
            InputEvent::MouseMoved(pos) => encoder.encode_v2(*pos),
            _ => js::value::null(encoder.guard),
        }
    }
}

struct JsEnvironment;


impl JsEnvironment {
    pub fn set_up(context: &js::Context , world: &mut World, prototypes: &HM) -> Self {
        let reference = unsafe{
            std::mem::transmute::<&mut World, &'static mut World>(world)
        };
        context.insert_user_data::<&mut World>(reference);

        insert(context, prototypes);
        Self{}
    }

    pub fn drop(self, context: &js::Context) {
        debug_assert!(context.remove_user_data::<&mut World>().is_some());

        debug_assert!(context.remove_user_data::<&HM>().is_some());
    }
}

impl ScriptingEngine for JsScriptEngine {
    type Controller = JsScript;
    type Config = JsEngineConfig;

    fn create(config: &Self::Config, world: &mut World) -> Self {
        world.resources.insert(ScriptCreationQueue{q:vec![]});
        let runtime = js::Runtime::new().unwrap();
        let context = js::Context::new(&runtime).unwrap();
        let ui_events_handler = EventDispatcher::<UiEvent>::create(world);
        let mut engine = Self {
            _runtime: runtime,
            context: ManuallyDrop::new(context),
            prototypes: Default::default(),
            ui_events_handler,
        };
        engine.configure(config);

        let nm = engine.register_namespace("dawg", None);
        engine.register_function("lolz", Some(&nm), |args: Vec<String>| {
            for item in args {
                print!("{}", item);

            }
            println!();
        });

        engine
    }

    fn create_script(&mut self, id: Entity, typ: &str, world: &mut World) {
        let command = format!("new {}();", typ);
        let env = JsEnvironment::set_up(&self.context, world, &self.prototypes);

        let guard = self.guard();

        let obj = js::script::eval(&guard, &command).unwrap();

        let prot = self.create_script_external(&guard, id);
        let obj = obj.into_object().unwrap();

        obj.set(&guard, js::Property::new(&guard, "go"), prot);

        let script = JsScript::new(obj, &guard);
        world.add_component(id, script);

        env.drop(&self.context)
    }

    fn update(&mut self,
              world: &mut World,

    ) {
        let guard = self.context.make_current().unwrap();

        let env = JsEnvironment::set_up(&self.context, world, &self.prototypes);


        let query = <(Read<JsScript>)>::query();

        self.ui_events_handler.dispatch(&guard, world);

        for (_entity_id, script) in query.iter_entities_immutable(world) {
            match &script.update {
                None => (),
                Some(fun) => { 
                    use failure::Fail;
                    match fun.call_with_this(&guard, &script.js_object, &[]) {
                        Ok(..) => (),
                        Err(err) => {
                            println!("error: {:?}", err);
                            match err.backtrace() {
                                None => (),
                                Some(x) => {
                                println!("{:?}", x);
                                }
                            }
                        }
                    }
                }
            }

        }
        env.drop(&self.context);

        drop(guard);

        let to_create : Vec<_> = std::mem::replace(&mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q, vec![]);

        for data in to_create {
            self.create_script(data.object, &data.script_type, world);
        }

    }
}

#[derive(Copy, Clone, Debug)]
pub enum JsRuntimeError {
    NotEnoughParameters,
    WrongTypeParameter,
    ExpectedParameterNotPresent,
}

struct JsResultEncoder<'a> {
    guard: &'a ContextGuard<'a>,
}

impl<'a> crate::traits::ResultEncoder for JsResultEncoder<'a> {
    type ResultType = js::value::Value;

    fn empty(&mut self) -> Self::ResultType {
        js::value::null(&self.guard)
    }

    fn encode_float(&mut self, value: f32) -> Self::ResultType {
        js::value::Number::from_double(&self.guard, value as f64).into()
    }
}


struct JsParamSource<'a> {
    guard: &'a ContextGuard<'a>,
    params: js::value::function::CallbackInfo,
    world: &'a World,
    current: usize,
}

impl<'a> crate::traits::ParametersSource for JsParamSource<'a> {
    type ErrorType = JsRuntimeError;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType> {
        let value = self.params.arguments[self.current].clone().into_number().ok_or(JsRuntimeError::WrongTypeParameter)?.value_double() as f32;
        self.current +=1;
        Result::Ok(value)
    }

    fn read_all<T: FunctionParameter>(&mut self) -> Result<Vec<T>, Self::ErrorType> {
        if self.current >= self.params.arguments.len() {
            return Result::Ok(vec![]);
        }
        let mut args = Vec::with_capacity(self.params.arguments.len() - self.current);
        while self.current < self.params.arguments.len() {
            args.push(T::read(self)?);
        }
        Result::Ok(args)
    }

    fn read_formatted(&mut self) -> Result<String, Self::ErrorType> {
        let value = self.params.arguments[self.current].to_string(self.guard);
        self.current +=1;
        Result::Ok(value)
    }

    fn read_system_data<T: 'static + Send + Sync>(&mut self) -> Result<legion::resource::FetchMut<T>, Self::ErrorType> {
        Result::Ok(self.world.resources.get_mut::<T>().unwrap())
        
    }


    /*
    fn read_component<'b, T: 'b + Send + Sync>(&'b mut self, id: Entity) -> Result<&'b T, JsRuntimeError> {
        let external = self.params.this.into_external().ok_or(JsRuntimeError::WrongTypeParameter)?;
        let this = unsafe { external.value::<game_object_api::GameObjectData>() };
        let ctx = self.guard.context();

        let world = api_helpers::world(&ctx);
        let component = world.get_component::<T>(this.id).ok_or(JsRuntimeError::WrongTypeParameter)?;
        Result::Ok(&component)
    }
    */
}

impl<'a> JsParamSource<'a> {
    pub fn create(guard: &'a ContextGuard<'a>, params: js::value::function::CallbackInfo, world: &'a World) -> Self {
        Self {
            guard,
            params,
            current: 0,
            world
        }
    }

}

impl crate::traits::ScriptApiRegistry for JsScriptEngine {
    type Namespace = js::value::Object;
    type Type = js::value::Value;

    //type ParamEncoder = Para
    type ErrorType = JsRuntimeError;

    fn register_namespace(&mut self, name: &str, parent: Option<&Self::Namespace>) -> Self::Namespace {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();
        let par = match parent {
            Some(x) => x,
            None => &global,
        };
        let ns = js::value::Object::new(&guard);
        par.set(&guard, js::Property::new(&guard, name), ns.clone());
        ns
    }


    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<&Self::Namespace>, fc: F)
    where P: crate::traits::FunctionParameter,
          R: crate::traits::FunctionResult,
          F: 'static + Send + Sync + Fn(P)-> R {
        let guard = self.context.make_current().unwrap();
        let fun = js::value::Function::new(&guard, Box::new(move |gd, params|{
            let ctx = gd.context();
            let world = api_helpers::world(&ctx);
            let mut param_source = JsParamSource::create(gd, params, world);
            let ret = fc(P::read(&mut param_source).unwrap()); // map to js exception?
            drop(param_source);
            let mut enc = JsResultEncoder{guard: gd};
            Result::Ok(ret.into_script_value(&mut enc))
        }));
        let global = guard.global();

        let parent = match namespace {
            Some(x) => x,
            None => &global,
        };
        parent.set(&guard, js::Property::new(&guard, name), fun);
        
    }
     fn register_native_type<T>(&mut self, name: &str, namespace: Option<&Self::Namespace>) -> Self::Type {
        let guard = self.context.make_current().unwrap();
        js::value::null(&guard)
     }

     fn register_static_property<P1, P2, R, F1, F2>(
         &mut self,
         name: &str,
         namespace: Option<&Self::Namespace>,
         getter: Option<F1>,
         setter: Option<F2>)
     where P1: FunctionParameter,
           P2: FunctionParameter,
           R: FunctionResult,
           F1: 'static + Send + Sync + Fn(P1)->R,
           F2: 'static + Send + Sync + Fn(P2) {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();

        let property_object = js::value::Object::new(&guard);

        match getter {
            None => (),
            Some(func) => {
                let fun = js::value::Function::new(&guard, Box::new(move |gd, params|{
                    let ctx = gd.context();
                    let world = api_helpers::world(&ctx);
                    let mut param_source = JsParamSource::create(gd, params, world);
                    let ret = func(P1::read(&mut param_source).unwrap());
                    let mut enc = JsResultEncoder{guard: gd};
                    Result::Ok(ret.into_script_value(&mut enc))
                }));
                property_object.set(&guard, js::Property::new(&guard, "get"), fun);
            }
        }
        match setter {
            None => (),
            Some(func) => {
                let fun = js::value::Function::new(&guard, Box::new(move |gd, params|{
                    let ctx = gd.context();
                    let world = api_helpers::world(&ctx);
                    let mut param_source = JsParamSource::create(gd, params, world);
                    func(P2::read(&mut param_source).unwrap()); // map to js exception?
                    Result::Ok(js::value::null(gd))
                }));
                property_object.set(&guard, js::Property::new(&guard, "set"), fun);
            }
        }
        let par = match namespace {
            Some(x) => x,
            None => &global,
        };

        par.define_property(&guard, js::Property::new(&guard, name), property_object);
     }
}

struct JsApi<'a> {
    guard: &'a ContextGuard<'a>,
}
struct EventHandler {
    object: js::value::Object,
    handler: js::value::Function,
}

impl PartialEq for EventHandler {
    fn eq(&self, other: &Self ) -> bool {
        self.object.as_raw() == other.object.as_raw() && self.handler.as_raw() == other.handler.as_raw()
    }
}

impl Eq for EventHandler{}
impl std::hash::Hash for EventHandler {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.object.as_raw().hash(state);
        self.handler.as_raw().hash(state);
    }
}

use js::ContextGuard;

struct ParamEncoder<'a> {
    guard: &'a ContextGuard<'a>,
    prototypes: &'a HM

}

impl<'a> ParamEncoder<'a> {
    pub fn encode_float(value: f32, guard: &ContextGuard) -> js::value::Value {
        js::value::Number::from_double(guard, value as f64).into()
    }

    pub fn encode_vec2(value: crate::math::Vec2, guard: &ContextGuard, prototypes: &HM) -> js::value::Value {
        let ob = js::value::External::new(guard, Box::new(value));
        ob.set_prototype(guard, prototypes[&InternalTypes::Vec2].clone());
        ob.into()
    }

    pub fn encode_f32(&self, value: f32) -> js::value::Value {
        Self::encode_float(value, self.guard)
    }

    pub fn encode_v2(&self, value: crate::math::Vec2) -> js::value::Value {
        Self::encode_vec2(value, self.guard, self.prototypes)
    }

}

trait DispatchableEvent: Copy + Clone + Send + Sync {
    type Hash: std::cmp::Eq + std::hash::Hash + Copy + Clone;
    fn hash(&self) -> Self::Hash;
    fn invoke_params(&self, encoder: &ParamEncoder) -> js::value::Value;
}

struct EventDispatcher<T: 'static + DispatchableEvent> {
    reader_id: shrev::ReaderId<T>,
    handlers: HashMap<T::Hash, HashSet<EventHandler>>,
}

unsafe impl<T: 'static + DispatchableEvent> Send for EventDispatcher<T> {}
unsafe impl<T: 'static + DispatchableEvent> Sync for EventDispatcher<T> {}

impl<T: 'static + DispatchableEvent> EventDispatcher<T> {
    pub fn create(world: &mut World) -> Self {
        let reader_id = world.resources.get_mut::<shrev::EventChannel<T>>().unwrap().register_reader();
        Self {
            reader_id,
            handlers: HashMap::default(),
        }
    }
    pub fn register(&mut self, event: T, handler: EventHandler) {
        let hash = event.hash();
        if !self.handlers.contains_key(&hash) {
            self.handlers.insert(hash, HashSet::new());
        }
        self.handlers.get_mut(&hash).unwrap().insert(handler);
    }

    pub fn deregister(&mut self, event: T, handler: EventHandler) {
        let hash = event.hash();
        match self.handlers.get_mut(&hash) {
            None => (),
            Some(mut x) => {
                x.remove(&handler);
            }
        }
    }

    fn dispatch(&mut self, guard: &js::ContextGuard, world: &mut World) {
        let mut channel = world.resources.get_mut::<shrev::EventChannel<T>>().unwrap(); 
        let events = channel.read(&mut self.reader_id);
        for event in events {
            let hash = event.hash();
            match self.handlers.get_mut(&hash) {
                None => (),
                Some(x) =>{
                    for hd in x.iter() {
                        hd.handler.call_with_this(guard, &hd.object, &[]);

                    }

                }
            }
        }
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
