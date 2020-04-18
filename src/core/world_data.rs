use crate::traits::{MeshId, TextureId, World, RenderingData, Controller, Value};

use std::cell::RefCell;
use crate::math::*;
use legion::prelude::*;

use super::game_object::{GameObject, ObjectsToDelete};
use super::transform::Transform;
use super::hierarchy::TransformHierarchy;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mesh {
    pub mesh_id: MeshId,
    pub texture_id: Option<TextureId>,
}

pub struct WorldData {
    pub wor: legion::prelude::World,
}

#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone)]
pub struct ViewportData(pub f32);

impl WorldData {
    pub fn new() -> WorldData {
        let mut wor = legion::prelude::World::new();
        wor.resources.insert(ObjectsToDelete::new());
        wor.resources.insert(ViewportData(10.0f32));
        wor.resources.insert(CameraData{position: Vec3::zeros(), rotation: Quat::identity()});
        WorldData {
            wor,
        }
    }
}

impl World for WorldData {
    fn get_legion(&mut self) -> &mut legion::prelude::World{
        &mut self.wor
    }

    fn set_renderable(&mut self, id: Entity, mesh: MeshId){
        self.add_renderable(id, Mesh{mesh_id: mesh, texture_id: None});
    }

    fn set_camera_position(&mut self, new_pos: Vec3) {
        // println!("setting camera_pos to {:?}", new_pos);
        self.wor.resources.get_mut::<CameraData>().unwrap().position = new_pos;
    }
}

unsafe impl Send for WorldData{}
unsafe impl Sync for WorldData{}

impl WorldData {
    pub fn add_renderable(&mut self, id: Entity,mesh: Mesh){
        self.wor.add_component(id, mesh);
    }
    pub fn exists(&self, id: Entity) -> bool {
        self.wor.is_alive(id)
    }
}

use crate::math::Matrix;

impl RenderingData for WorldData {
    fn get_projection_matrix(&self) -> Matrix {
        let viewport_height = self.wor.resources.get::<ViewportData>().unwrap().0;
        let top = viewport_height / 2.0f32;
        let bot = - top;
        let right = 1920.0f32 / 1080.0f32 * top;
        let left = -right;
        let near = -50.0f32;
        let far = 300.0f32;
        let mut temp = nalgebra_glm::ortho_lh_zo(left, right, bot, top, near, far);
        // let mut temp = nalgebra_glm::perspective_lh_zo(
        //     256.0f32 / 108.0, f32::to_radians(45.0f32), 0.1f32, 100.0f32);
        temp[(1, 1)] *= -1.0;
        temp
    }

    fn get_view_matrix(&self) -> Matrix {
        let camera_data = self.wor.resources.get::<CameraData>().unwrap();

        nalgebra_glm::quat_to_mat4(&camera_data.rotation) * nalgebra_glm::translation(&(-camera_data.position))
    }

    fn get_rendering_constant(&self, name: &str) -> Value {
        match name {
            "projection_mat" => Value::Matrix4(self.get_projection_matrix()),
            "view_mat" => Value::Matrix4(self.get_view_matrix()),
            "lightColor" => Value::Vector3(Vec3::new(0.6f32, 0.6f32, 0.6f32)),
            "lightDir" => Value::Vector3(Vec3::new(2.0f32, 1.0f32, -0.1f32)),
            "camera_pos" => Value::Vector3(self.wor.resources.get::<CameraData>().unwrap().position),
            _ => Value::None,
        }

    }

    fn get_renderables(
        &self,
        buffer: Option<Vec<(MeshId, Option<TextureId>, Matrix)>>
    ) -> Vec<(MeshId, Option<TextureId>, Matrix)> {
        use legion::prelude::*;

        let query = <(Read<Mesh>)>::query();


        let mut buf = match buffer {
            Some(mut vec) => {
                // if vec.len() < self.renderables.len() {
                //     vec.reserve(self.renderables.len() - vec.len());
                // }
                vec.clear();
                vec
            }
            None => Vec::new(),
        };

        for (entity_id, mesh) in query.iter_entities_immutable(&self.wor) {
            let mat = TransformHierarchy::get_global_matrix(&self.wor, entity_id);
            buf.push((mesh.mesh_id, mesh.texture_id, mat));
        }
        buf
    }
}
