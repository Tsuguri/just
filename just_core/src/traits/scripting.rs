use serde::Deserialize;

use legion::entity::Entity;
use legion::world::World as LWorld;

pub mod function_params;
mod function_result;

pub use function_params::{FunctionParameter, ParametersSource};
pub use function_result::{FunctionResult, ResultEncoder};

#[derive(Debug)]
pub enum TypeCreationError {
    TypeAlreadyRegistered,
    TypeNotRegistered,
}

pub trait ScriptingEngine: Sized {
    // + ScriptApiRegistry {
    type Config: Deserialize<'static>;
    type SAR: ScriptApiRegistry;

    fn create<Builder: FnOnce(&mut Self::SAR)>(config: Self::Config, world: &mut LWorld, api_builder: Builder) -> Self;

    fn create_script(&mut self, gameobject_id: Entity, typ: &str, world: &mut LWorld);

    fn update(&mut self, world: &mut LWorld);
}

pub trait ScriptApiRegistry {
    type Namespace;
    type Type;
    type NativeType;

    type ErrorType;

    fn register_namespace(&mut self, name: &str, parent: Option<&Self::Namespace>) -> Self::Namespace;

    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<&Self::Namespace>, fc: F)
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R;

    fn register_native_type<T, P, F>(
        &mut self,
        name: &str,
        namespace: Option<&Self::Namespace>,
        constructor: F,
    ) -> Result<Self::NativeType, TypeCreationError>
    where
        T: 'static,
        P: FunctionParameter,
        F: 'static + Send + Sync + Fn(P) -> T;

    fn get_native_type<T: 'static>(&mut self) -> Option<Self::NativeType>;

    fn register_component<T, F>(
        &mut self,
        name: &str,
        namespace: Option<&Self::Namespace>,
        constructor: F,
    ) -> Result<Self::NativeType, TypeCreationError>
    where
        T: 'static + Send + Sync,
        F: 'static + Send + Sync + Fn() -> T;

    fn register_native_type_method<P, R, F>(
        &mut self,
        _type: &Self::NativeType,
        name: &str,
        method: F,
    ) -> Result<(), TypeCreationError>
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R;

    fn register_native_type_property<P1, P2, R, F1, F2>(
        &mut self,
        _type: &Self::NativeType,
        name: &str,
        getter: Option<F1>,
        setter: Option<F2>,
    ) where
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
        setter: Option<F2>,
    ) where
        P1: FunctionParameter,
        P2: FunctionParameter,
        R: FunctionResult,
        F1: 'static + Send + Sync + Fn(P1) -> R,
        F2: 'static + Send + Sync + Fn(P2);
}
