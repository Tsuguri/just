use just_rendyocto::resources::{MeshId, TextureId};

use just_core::ecs::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Renderable {
    pub mesh: Option<MeshId>,
    pub texture: Option<TextureId>,
}

impl std::default::Default for Renderable {
    fn default() -> Self {
        Renderable {
            mesh: None,
            texture: None,
        }
    }
}

//#[derive(Copy, Clone, Debug, PartialEq)]
//pub struct Mesh {
    //pub mesh_id: MeshId,
    //pub texture_id: Option<TextureId>,
//}

impl Renderable {
    pub fn add_renderable_to_go(world: &mut World, id: Entity, mesh: MeshId) {
        world.add_component(
            id,
            Renderable {
                mesh: Some(mesh),
                texture: None,
            },
        );
    }
    pub fn add_tex_renderable(world: &mut World, id: Entity, mesh: Renderable) {
        world.add_component(id, mesh);
    }
}
