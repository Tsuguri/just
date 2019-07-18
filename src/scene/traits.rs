use serde::Deserialize;

pub trait ResourceManager {
    type Config: Deserialize<'static>;
    type MeshId: Copy + Clone;
    type TextureId: Copy + Clone;
    //type MeshId: Copy + Clone;


    fn get_mesh(&self, name: &str) -> Option<Self::MeshId>;
    fn get_texture(&self, name: &str) -> Option<Self::TextureId>;
    fn load_resources(&mut self, config: &Self::Config);
    fn create(config: &Self::Config) -> Self;

}
pub use super::GameObjectId;

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

pub trait Renderer<H: Hardware+ ?Sized> {
    fn create(hardware: &mut H)-> Self;
    fn run(&mut self, hardware: &mut H, res: &H::RM);
    fn dispose(&mut self, hardware: &mut H);
}

pub trait Hardware {
    type RM: ResourceManager;
    type Renderer: Renderer<Self>;
    type Config: Deserialize<'static>;

    fn create(config: &Self::Config) -> Self;

}