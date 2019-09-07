use serde::Deserialize;

pub type MeshId = usize;
pub type TextureId = usize;

pub trait ResourceManager<HW: Hardware + ?Sized> {
    type Config: Deserialize<'static>;


    fn get_mesh(&self, name: &str) -> Option<MeshId>;
    fn get_texture(&self, name: &str) -> Option<TextureId>;
    fn load_resources(&mut self, config: &Self::Config, hardware: &mut HW);
    fn create(config: &Self::Config, hardware: &mut HW) -> Self;
}

pub use super::GameObjectId;
use super::math::*;
use std::sync::Arc;

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
    fn set_gameobject_property(&mut self, name: &str, value: GameObjectId);
}


pub trait ScriptingEngine: Sized {
    type Controller: Controller + 'static;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;
    //fn set_world_data<HW: Hardware>(&self, world_data: &mut super::WorldData<HW>);

    fn create_script(&mut self, gameobject_id: GameObjectId, typ: &str) -> Self::Controller;

    fn update<HW: Hardware + 'static, RM: ResourceManager<HW> + Send + Sync + 'static>(&mut self,
              world: &mut crate::scene::WorldData,
              scripts: &mut super::Data<Self::Controller>,
              resources: &RM,
              keyboard: &crate::input::KeyboardState,
              mouse: &crate::input::MouseState,
    );
}


pub trait Data {
    fn get_projection_matrix(&self) -> Matrix;
    fn get_view_matrix(&self) -> Matrix;

    fn get_renderables(
        &self,
        buffer: Option<Vec<(MeshId, Option<TextureId>, Matrix)>>,
    ) -> Vec<(MeshId, Option<TextureId>, Matrix)>;
}

pub trait Renderer<H: Hardware + ?Sized> {
    fn create(hardware: &mut H, world: &(Data + 'static), res: Arc<H::RM>) -> Self;
    fn run(&mut self, hardware: &mut H, res: &H::RM, world: &(Data + 'static));
    fn dispose(&mut self, hardware: &mut H, world: &(Data + 'static));
}

pub trait Hardware {
    type RM: ResourceManager<Self>;
    type Renderer: Renderer<Self>;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;
}