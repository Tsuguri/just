use rendy::{
    mesh::Mesh,
};

use super::Backend;
use crate::scene::traits::ResourceManager as RMTrait;

pub struct ResourceManager {
    meshes: Vec<Mesh<Backend>>,
    textures: Vec<rendy::texture::Texture<Backend>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self{
            meshes: vec![],
            textures: vec![],
        }
    }
}

impl RMTrait for ResourceManager {
    type Config = String;
    type MeshId = usize;
    type TextureId = usize;


    fn get_mesh(&self, name: &str) -> Option<Self::MeshId> {
        None
    }
    fn get_texture(&self, name: &str) -> Option<Self::TextureId> {
        None
    }
    fn load_resources(&mut self, config: &Self::Config){
        let path = std::path::Path::new(config);
        println!("Loading resources from: {}", std::fs::canonicalize(path).unwrap().display());

    }

    fn create(config: &Self::Config) -> Self {
        let mut s : Self = Default::default();
        s.load_resources(config);
        s
    }
}