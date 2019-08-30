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
    fn create(_hardware: &mut MockHardware, _world: &Data<MockHardware>, _res: Arc<MockResourceManager>) -> Self {
        Self {}
    }
    fn run(&mut self, _hardware: &mut MockHardware, _res: &MockResourceManager, _world: &Data<MockHardware>) {}
    fn dispose(&mut self, _hardware: &mut MockHardware, _world: &Data<MockHardware>){}
}

impl ResourceManager<MockHardware> for MockResourceManager {
    type Config = i32;
    type MeshId = usize;
    type TextureId = usize;


    fn get_mesh(&self, _name: &str) -> Option<Self::MeshId> {
        None
    }
    fn get_texture(&self, _name: &str) -> Option<Self::TextureId> {
        None
    }
    fn load_resources(&mut self, _config: &Self::Config, _hardware: &mut MockHardware) {}

    fn create(_config: &Self::Config, _hardware: &mut MockHardware) -> Self {
        Self {}
    }
}