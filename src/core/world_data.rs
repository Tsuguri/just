use crate::traits::{MeshId, TextureId, World, RenderingData, Controller, Value};

use std::cell::RefCell;
use crate::math::*;
use legion::prelude::*;

use super::game_object::{GameObject};
use super::transform::Transform;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Mesh {
    pub mesh_id: MeshId,
    pub texture_id: Option<TextureId>,
}

pub struct WorldData {
    // at the same time indicates if object is active
    pub wor: legion::prelude::World,
}

#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone)]
pub struct ViewportData(pub f32);

#[derive(Clone)]
pub struct ObjectsToDelete(Vec<Entity>);

impl WorldData {
    pub fn new() -> WorldData {
        let mut wor = legion::prelude::World::new();
        wor.resources.insert(ObjectsToDelete(Vec::new()));
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

    fn get_name(&self, id: Entity) -> String{
        self.wor.get_component::<GameObject>(id).unwrap().name.clone()
    }

    fn set_name(&mut self, id: Entity, name: String){
        self.wor.get_component_mut::<GameObject>(id).unwrap().name = name;
    }

    fn set_local_pos(&mut self, id: Entity, new_position: Vec3) -> Result<(), ()>{
        self.set_local_position(id, new_position);
        Result::Ok(())
    }
    fn get_local_pos(&self, id: Entity) -> Result<Vec3, ()>{
        Result::Ok(self.get_local_position(id))
    }

    fn get_global_pos(&self, id: Entity) -> Result<Vec3, ()>{
        Result::Ok(self.get_global_position(id))
    }

    fn set_local_sc(&mut self, id: Entity, new_scale: Vec3) -> Result<(), ()>{
        self.set_local_scale(id, new_scale);
        Result::Ok(())
    }
    fn get_local_sc(&self, id: Entity) -> Result<Vec3, ()>{
        Result::Ok(self.get_local_scale(id))
    }

    fn get_parent(&self, id: Entity) -> Option<Entity>{
        self.wor.get_component::<GameObject>(id).unwrap().parent
    }

    fn set_parent(&mut self, obj: Entity, new_parent: Option<Entity>) -> Result<(),()>{
        if !self.exists(obj) {
            return Result::Err(());
        }
        match new_parent {
            Some(x) => {
                if !self.exists(x) {
                    return Result::Err(());
                }
                self.wor.get_component_mut::<GameObject>(x).unwrap().children.push(obj);
            }
            None => (),
        }
        let parent = self.wor.get_component::<GameObject>(obj).unwrap().parent;
        match parent {
            None => (),
            Some(x) => {
                let mut data = self.wor.get_component_mut::<GameObject>(x).unwrap();
                let index = data.children.iter().position(|y| *y == obj).unwrap();
                data.children.remove(index);
            }
        }
        let mut data = self.wor.get_component_mut::<GameObject>(obj).unwrap();
        data.parent = new_parent;
        drop(data);
        self.void_local_matrix(obj);

        Result::Ok(())
    }

    fn find_by_name(&self, name: &str) -> Vec<Entity>{
        legion::prelude::Read::<GameObject>::query().iter_entities_immutable(&self.wor).filter(|(x, y)| {
            y.name == name
        }).map(|(x,_y)| x).collect()
    }

    fn create_gameobject(&mut self) -> Entity {
        let go = GameObject::new();

        let ent_id = self.wor.insert(
            (),
            vec![
                (Transform::new(),go,),
            ],
        ).to_vec();
        ent_id[0]
    }

    fn destroy_gameobject(&mut self, id: Entity) {
        self.wor.resources.get_mut::<ObjectsToDelete>().unwrap().0.push(id);
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

    pub fn remove_marked(&mut self) {
        let mut to_destroy = self.wor.resources.get_mut::<ObjectsToDelete>().unwrap();
        let objects = std::mem::replace(&mut to_destroy.0, vec![]);
        drop(to_destroy);
        for obj in objects.into_iter() {
            // might have been removed as child of other object
            if !self.exists(obj) {
                continue;
            }
            self.remove_game_object(obj);
        }
    }


    pub fn remove_game_object(&mut self, id: Entity) {
        let data = (*self.wor.get_component::<GameObject>(id).unwrap()).clone();
        for child in data.children {
            self.remove_game_object(child);
        }
        self.remove_single(id);
    }

    fn remove_single(&mut self, id: Entity) {
        self.wor.delete(id);
        self.set_parent(id, None).unwrap();
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
            let mat = self.get_global_matrix(entity_id);
            buf.push((mesh.mesh_id, mesh.texture_id, mat));
        }
        buf
    }
}
