use crate::scene::traits::*;
use std::sync::Arc;

pub struct MockResourceManager {}

pub struct MockHardware {}

pub struct MockRenderer {}


impl Hardware for MockHardware {
    type RM = MockResourceManager;
    type Renderer = MockRenderer;
    type Config = i32;

    fn create(config: &Self::Config) -> Self {
        Self {}
    }
}

impl Renderer<MockHardware> for MockRenderer {
    fn create(hardware: &mut MockHardware, world: &Data, res: Arc<MockResourceManager>) -> Self {
        Self {}
    }
    fn run(&mut self, hardware: &mut MockHardware, res: &MockResourceManager, world: &Data) {}
    fn dispose(&mut self, hardware: &mut MockHardware, world: &Data){}
}

impl ResourceManager<MockHardware> for MockResourceManager {
    type Config = i32;
    type MeshId = usize;
    type TextureId = usize;


    fn get_mesh(&self, name: &str) -> Option<Self::MeshId> {
        None
    }
    fn get_texture(&self, name: &str) -> Option<Self::TextureId> {
        None
    }
    fn load_resources(&mut self, config: &Self::Config, hardware: &mut MockHardware) {}

    fn create(config: &Self::Config, _hardware: &mut MockHardware) -> Self {
        Self {}
    }
}