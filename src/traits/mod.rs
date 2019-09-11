use std::sync::Arc;

use serde::Deserialize;

use crate::math::*;

pub type MeshId = usize;
pub type TextureId = usize;

slotmap::new_key_type!(pub struct GameObjectId;);

pub type Map<T> = slotmap::HopSlotMap<GameObjectId, T>;
pub type Data<T> = slotmap::SecondaryMap<GameObjectId, T>;

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
    fn set_gameobject_property(&mut self, name: &str, value: GameObjectId);
}

pub trait World: Send + Sync {
    fn get_name(&self, id: GameObjectId) -> String;

    fn set_name(&mut self, id: GameObjectId, name: String);
    fn set_local_pos(&mut self, id: GameObjectId, new_position: Vec3) -> Result<(), ()>;
    fn get_local_pos(&self, id: GameObjectId) -> Result<Vec3, ()>;

    fn get_parent(&self, id: GameObjectId) -> Option<GameObjectId>;
    fn set_parent(&mut self, id: GameObjectId, new_parent: Option<GameObjectId>) -> Result<(), ()>;

    fn find_by_name(&self, name: &str) -> Vec<GameObjectId>;

    fn create_gameobject(&mut self) -> GameObjectId;

    fn destroy_gameobject(&mut self, id: GameObjectId);

    fn set_renderable(&mut self, id: GameObjectId, mesh: MeshId);
}


pub trait ScriptingEngine: Sized {
    type Controller: Controller + 'static;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;

    fn create_script(&mut self, gameobject_id: GameObjectId, typ: &str) -> Self::Controller;

    fn update(&mut self,
              world: &mut dyn World,
              scripts: &mut Data<Self::Controller>,
              resources: &ResourceProvider,
              keyboard: &crate::input::KeyboardState,
              mouse: &crate::input::MouseState,
              current_time: f64,
    );
}


pub trait RenderingData {
    fn get_projection_matrix(&self) -> Matrix;
    fn get_view_matrix(&self) -> Matrix;

    fn get_renderables(
        &self,
        buffer: Option<Vec<(MeshId, Option<TextureId>, Matrix)>>,
    ) -> Vec<(MeshId, Option<TextureId>, Matrix)>;
}

pub trait Renderer<H: Hardware + ?Sized> {
    fn create(hardware: &mut H, world: &(dyn RenderingData + 'static), res: Arc<H::RM>) -> Self;
    fn run(&mut self, hardware: &mut H, res: &H::RM, world: &(dyn RenderingData + 'static));
    fn dispose(&mut self, hardware: &mut H, world: &(dyn RenderingData + 'static));
}

pub trait Hardware {
    type RM: ResourceManager<Self>;
    type Renderer: Renderer<Self>;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;
}