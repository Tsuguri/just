use std::sync::Arc;

use serde::Deserialize;

use legion::prelude::{Entity, World};

mod function_params;
mod function_result;

pub use function_params::*;
pub use function_result::*;

pub type MeshId = usize;
pub type TextureId = usize;

pub trait ResourceProvider: Send + Sync {
    fn get_mesh(&self, name: &str) -> Option<MeshId>;
    fn get_texture(&self, name: &str) -> Option<TextureId>;
}

pub trait ResourceManager<HW: Hardware + ?Sized>: ResourceProvider {
    type Config: Deserialize<'static>;

    fn load_resources(&mut self, config: &Self::Config, hardware: &mut HW);
    fn create(config: &Self::Config, hardware: &mut HW) -> Self;
}

pub trait Controller {
    fn prepare(&mut self);
    fn init(&mut self);
    fn destroy(&mut self);

    fn get_type_name(&self) -> String;

    fn set_bool_property(&mut self, name: &str, value: bool);
    fn set_int_property(&mut self, name: &str, value: i64);
    fn set_float_property(&mut self, name: &str, value: f32);
    fn set_string_property(&mut self, name: &str, value: String);

    fn set_controller_property(&mut self, name: &str, value: &Self);
    fn set_gameobject_property(&mut self, name: &str, value: Entity);
}

pub trait ScriptingEngine: Sized {
    type Controller: Controller + 'static;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config, world: &mut World) -> Self;

    fn create_script(&mut self, gameobject_id: Entity, typ: &str, world: &mut World);

    fn update(&mut self,
              world: &mut World,
    );
}

#[derive(Debug)]
pub enum TypeCreationError {
    TypeAlreadyRegistered,
    TypeNotRegistered,
}

pub trait ScriptApiRegistry {
    type Namespace;
    type Type;
    type NativeType;

    type ErrorType;

    fn register_namespace(&mut self, name: &str, parent: Option<&Self::Namespace>) -> Self::Namespace;

    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<&Self::Namespace>, fc: F)
        where P: FunctionParameter,
              R: FunctionResult,
              F: 'static + Send + Sync + Fn(P) -> R;

    fn register_native_type<T, P, F>(&mut self, name: &str, namespace: Option<&Self::Namespace>, constructor: F) -> Result<Self::NativeType, TypeCreationError>
        where T: 'static,
              P: FunctionParameter,
              F: 'static + Send + Sync + Fn(P) -> T;

    fn register_native_type_method<T, P, R, F>(&mut self, name: &str, method: F) -> Result<(), TypeCreationError>
        where T: 'static,
              P: FunctionParameter,
              R: FunctionResult,
              F: 'static + Send + Sync + Fn(P) -> R;

    fn register_native_type_property<T, P1, P2, R, F1, F2>(
        &mut self,
        name: &str,
        getter: Option<F1>,
        setter: Option<F2>)
        where T: 'static,
              P1: FunctionParameter,
              P2: FunctionParameter,
              R: FunctionResult,
              F1: 'static + Send + Sync + Fn(P1) -> R,
              F2: 'static + Send + Sync + Fn(P2);

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
              F2: 'static + Send + Sync + Fn(P2);

    fn register_component<T: 'static>(&mut self, name: &str);
}

pub trait Renderer<H: Hardware + ?Sized> {
    fn create(hardware: &mut H, world: &mut World, res: Arc<H::RM>) -> Self;
    fn run(&mut self, hardware: &mut H, res: &H::RM, world: &World);
    fn dispose(&mut self, hardware: &mut H, world: &World);
}

pub trait Hardware {
    type RM: ResourceManager<Self>;
    type Renderer: Renderer<Self>;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;
}
