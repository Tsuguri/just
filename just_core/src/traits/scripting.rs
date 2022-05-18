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

#[derive(Copy, Clone, Debug)]
pub enum RuntimeError {
    NotEnoughParameters,
    WrongTypeParameter,
    ExpectedParameterNotPresent,
    TypeNotRegistered,
    ComponentNotPresent,
}

pub type NamespaceId = i32;
pub type NativeTypeId = i32;

pub trait ScriptApiRegistry<'a, 'b> {
    fn register_namespace(&mut self, name: &str, parent: Option<NamespaceId>) -> NamespaceId;

    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<NamespaceId>, fc: F)
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R;

    fn register_native_type<T, P, F>(
        &mut self,
        name: &str,
        namespace: Option<NamespaceId>,
        constructor: F,
    ) -> Result<NativeTypeId, TypeCreationError>
    where
        T: 'static,
        P: FunctionParameter,
        F: 'static + Send + Sync + Fn(P) -> T;

    fn get_native_type<T: 'static>(&mut self) -> Option<NativeTypeId>;

    fn register_component<T, F>(
        &mut self,
        name: &str,
        namespace: Option<NamespaceId>,
        constructor: F,
    ) -> Result<NativeTypeId, TypeCreationError>
    where
        T: 'static + Send + Sync,
        F: 'static + Send + Sync + Fn() -> T;

    fn register_native_type_method(
        &mut self,
        _type: NativeTypeId,
        name: &str,
        fc: impl v8::MapFnTo<v8::FunctionCallback>,
    ) -> Result<(), TypeCreationError>;

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
        F2: 'static + Send + Sync + Fn(P2);

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
        F2: 'static + Send + Sync + Fn(P2);
}
