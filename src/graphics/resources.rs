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
        Self {
            meshes: vec![],
            textures: vec![],
        }
    }
}

impl RMTrait<super::Hardware> for ResourceManager {
    type Config = String;
    type MeshId = usize;
    type TextureId = usize;


    fn get_mesh(&self, name: &str) -> Option<Self::MeshId> {
        None
    }
    fn get_texture(&self, name: &str) -> Option<Self::TextureId> {
        None
    }
    fn load_resources(&mut self, config: &Self::Config, hardware: &mut super::Hardware) {
        let path = std::path::Path::new(config);
        println!("Loading resources from: {}", std::fs::canonicalize(path).unwrap().display());
        let paths = std::fs::read_dir(path).map_err(|err| "counldn't read directory").unwrap();
        for path in paths {
            println!("reading thing: {:?}", path);
            let path = path.unwrap().path();
            if path.extension().unwrap().to_str() == Some("obj") {
                self.load_model(hardware, &path);
            }
        }
    }

    fn create(config: &Self::Config, hardware: &mut super::Hardware) -> Self {
        let mut s: Self = Default::default();
        s.load_resources(config, hardware);
        s
    }
}

impl ResourceManager {
    fn load_model(&mut self, hardware: &mut super::Hardware, filename: &std::path::PathBuf) {
        println!("loading model: {:?}", filename);
        let bytes = std::fs::read(filename).unwrap();
        let mut obj_builder = rendy::mesh::obj::load_from_obj(&bytes);
        let mut obj_builder = match obj_builder {
            Ok(x) => x,
            Err(y) => {
                println!("Error: {:?}", y);
                return;
            }
        };
        if obj_builder.len() > 1 {
            println!("model {:?} contains more than one object", filename);
            return;
        }
        let model_builder = obj_builder.pop().unwrap();
        let qid = hardware.families.family(hardware.used_family).queue(0).id();
        let model = model_builder.0.build(qid, &hardware.factory).unwrap();
    }
}