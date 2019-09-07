use crate::scene::traits::*;
use std::sync::Arc;

pub struct MockResourceManager {}

pub struct MockHardware {}

pub struct MockRenderer {}


impl Hardware for MockHardware {
    type RM = MockResourceManager;
    type Renderer = MockRenderer;
    type Config = i32;

    fn create(_config: &Self::Config) -> Self {
        Self {}
    }
}

impl Renderer<MockHardware> for MockRenderer {
    fn create(_hardware: &mut MockHardware, _world: &Data, _res: Arc<MockResourceManager>) -> Self {
        Self {}
    }
    fn run(&mut self, _hardware: &mut MockHardware, _res: &MockResourceManager, _world: &Data) {}
    fn dispose(&mut self, _hardware: &mut MockHardware, _world: &Data){}
}

impl ResourceManager<MockHardware> for MockResourceManager {
    type Config = i32;


    fn get_mesh(&self, _name: &str) -> Option<MeshId> {
        None
    }
    fn get_texture(&self, _name: &str) -> Option<TextureId> {
        None
    }
    fn load_resources(&mut self, _config: &Self::Config, _hardware: &mut MockHardware) {}

    fn create(_config: &Self::Config, _hardware: &mut MockHardware) -> Self {
        Self {}
    }
}