use std::sync::Arc;

use serde::Deserialize;

use just_core::ecs::prelude::World as LWorld;
use crate::graphics::Hw;

pub type MeshId = usize;
pub type TextureId = usize;

pub trait ResourceProvider: Send + Sync {
    fn get_mesh(&self, name: &str) -> Option<MeshId>;
    fn get_texture(&self, name: &str) -> Option<TextureId>;
}
