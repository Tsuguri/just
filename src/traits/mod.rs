use std::sync::Arc;

use serde::Deserialize;

use crate::math::*;
use legion::prelude::{Entity, World};

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
              current_time: f64,
    );
}

pub trait ParametersSource {
    type ErrorType;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType>;

    fn read_formatted(&mut self) -> Result<String, Self::ErrorType>;

    fn read_all<T: FunctionParameter>(&mut self) -> Result<Vec<T>, Self::ErrorType>;
}

pub trait FunctionParameter: Sized {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType>;
}

impl FunctionParameter for f32 {
    fn read<PS: ParametersSource>(source: &mut PS)-> Result<Self, PS::ErrorType> {
        source.read_float()
    }
}

impl FunctionParameter for () {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        Result::Ok(())
    }
}

impl<T: FunctionParameter> FunctionParameter for Vec<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_all::<T>()
    }
}

impl FunctionParameter for String {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<String, PS::ErrorType> {
        source.read_formatted()
    }
}

pub trait ScriptApiRegistry {
    type Namespace;
    type Type;

    //type ParamEncoder;
    type ErrorType;

    fn register_namespace(&mut self, name: &str, parent: Option<&Self::Namespace>) -> Self::Namespace;

    fn register_function<P, F>(&mut self, name: &str, namespace: Option<&Self::Namespace>, fc: F)
        where P: FunctionParameter,
              F: 'static + Send + Sync + Fn(P);

    fn register_native_type<T>(&mut self, name: &str, namespace: Option<&Self::Namespace>) -> Self::Type;
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
