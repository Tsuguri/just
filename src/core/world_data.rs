use crate::traits::{MeshId, TextureId};

use legion::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mesh {
    pub mesh_id: MeshId,
    pub texture_id: Option<TextureId>,
}

impl Mesh {
    pub fn add_renderable_to_go(world: &mut legion::prelude::World, id: Entity, mesh: MeshId) {
        world.add_component(
            id,
            Mesh {
                mesh_id: mesh,
                texture_id: None,
            },
        );
    }
    pub fn add_tex_renderable(world: &mut legion::prelude::World, id: Entity, mesh: Mesh) {
        world.add_component(id, mesh);
    }
}
