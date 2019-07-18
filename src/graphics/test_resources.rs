use crate::scene::traits::*;

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
    fn create(hardware: &mut MockHardware) -> Self {
        Self {}
    }
    fn run(&mut self, hardware: &mut MockHardware, res: &MockResourceManager) {}
    fn dispose(&mut self, hardware: &mut MockHardware){}
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

    fn create(config: &Self::Config, hardware: &mut MockHardware) -> Self {
        Self {}
    }
}