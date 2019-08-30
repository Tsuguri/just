use serde::Deserialize;

pub trait ResourceManager<HW: Hardware + ?Sized> {
    type Config: Deserialize<'static>;
    type MeshId: Copy + Clone;
    type TextureId: Copy + Clone;
    //type MeshId: Copy + Clone;


    fn get_mesh(&self, name: &str) -> Option<Self::MeshId>;
    fn get_texture(&self, name: &str) -> Option<Self::TextureId>;
    fn load_resources(&mut self, config: &Self::Config, hardware: &mut HW);
    fn create(config: &Self::Config, hardware: &mut HW) -> Self;

}

pub type MeshId<HW: Hardware> = <HW::RM as ResourceManager<HW>>::MeshId;
pub type TextureId<HW: Hardware> = <HW::RM as ResourceManager<HW>>::TextureId;

pub use super::GameObjectId;
use super::math::*;
use std::sync::Arc;

pub trait Controller {
    fn prepare(&mut self);
    fn update(&mut self);
    fn init(&mut self);
    fn destroy(&mut self);

    fn get_type_name(&self) -> String;

    fn set_bool_property(&mut self, name: &str, value: bool);
    fn set_int_property(&mut self, name: &str, value: i64);
    fn set_float_property(&mut self, name: &str, value: f32);
    fn set_string_property(&mut self, name: &str, value: String);

    fn set_controller_property(&mut self, name: &str, value: &Self);
    fn set_gameobject_property(&mut self, name: &str, value: GameObjectId);
}


pub trait ScriptingEngine {
    type Controller: Controller;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) ->Self;
}


pub trait Data<HW: Hardware> {
    fn get_projection_matrix(&self) -> Matrix;
    fn get_view_matrix(&self) -> Matrix;

    fn get_renderables(
        &self,
        buffer: Option<Vec<(MeshId<HW>, Option<TextureId<HW>>, Matrix)>>
    ) -> Vec<(MeshId<HW>, Option<TextureId<HW>>, Matrix)>;
}

pub trait Renderer<H: Hardware+ ?Sized> {
    fn create(hardware: &mut H, world: &(Data<H>+ 'static), res: Arc<H::RM>)-> Self;
    fn run(&mut self, hardware: &mut H, res: &H::RM, world: &(Data<H> + 'static));
    fn dispose(&mut self, hardware: &mut H, world: &(Data<H> + 'static));
}

pub trait Hardware {
    type RM: ResourceManager<Self>;
    type Renderer: Renderer<Self>;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;

}