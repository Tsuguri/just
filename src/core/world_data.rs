use crate::traits::{Data, Map, MeshId, TextureId, World, GameObjectId, RenderingData, Controller, Value};

use std::cell::RefCell;
use crate::math::*;
use legion::prelude::*;

use super::game_object::{GameObject, Transform};

pub struct Mesh {
    pub mesh_id: MeshId,
    pub texture_id: Option<TextureId>,
}

pub struct WorldData<C: Controller> {
    // at the same time indicates if object is active
    pub objects: Map<bool>,
    pub object_data: Data<GameObject>,
    pub renderables: Data<Mesh>,
    pub scripts: Data<C>,
    pub to_destroy: Vec<GameObjectId>,
    pub other_id: Data<legion::prelude::Entity>,
    pub camera_position: Vec3,
    pub camera_rotation: Quat,
    pub viewport_height: f32,

    pub wor: legion::prelude::World,
}

impl<C: Controller>  World for WorldData<C> {
    fn get_name(&self, id: GameObjectId) -> String{
        self.object_data[id].name.clone()

    }

    fn set_name(&mut self, id: GameObjectId, name: String){
        self.object_data[id].name = name;
    }

    fn set_local_pos(&mut self, id: GameObjectId, new_position: Vec3) -> Result<(), ()>{
        self.set_local_position(id, new_position);
        Result::Ok(())
    }
    fn get_local_pos(&self, id: GameObjectId) -> Result<Vec3, ()>{
        Result::Ok(self.get_local_position(id))
    }

    fn set_local_sc(&mut self, id: GameObjectId, new_scale: Vec3) -> Result<(), ()>{
        self.set_local_scale(id, new_scale);
        Result::Ok(())
    }
    fn get_local_sc(&self, id: GameObjectId) -> Result<Vec3, ()>{
        Result::Ok(self.get_local_scale(id))
    }

    fn get_parent(&self, id: GameObjectId) -> Option<GameObjectId>{
        self.object_data[id].parent
    }

    fn set_parent(&mut self, obj: GameObjectId, new_parent: Option<GameObjectId>) -> Result<(),()>{
        if !self.exists(obj) {
            return Result::Err(());
        }
        match new_parent {
            Some(x) => {
                if !self.exists(x) {
                    return Result::Err(());
                }
                self.object_data[x].children.push(obj);
            }
            None => (),
        }
        match self.object_data[obj].parent {
            None => (),
            Some(x) => {
                let index = self.object_data[x].children.iter().position(|y| *y == obj).unwrap();
                self.object_data[x].children.remove(index);
            }
        }
        self.object_data[obj].parent = new_parent;
        self.void_local_matrix(obj);
        //self.object_data[obj].void_local_matrix(&self);

        Result::Ok(())
    }

    fn find_by_name(&self, name: &str) -> Vec<GameObjectId>{
        self.object_data.iter().filter(|(x, y)| {
            y.name == name
        }).map(|(x,y)| x).collect()
    }

    fn create_gameobject(&mut self) -> GameObjectId {
        let id = self.objects.insert(true);
        let go = GameObject::new(id);

        let tr = Transform::new();
        self.object_data.insert(id, go);
        let ent_id = self.wor.insert(
            (),
            vec![
                (Transform::new(),),
            ],
        ).to_vec();
        self.other_id.insert(id, ent_id[0]);
        id
    }

    fn destroy_gameobject(&mut self, id: GameObjectId) {
        self.to_destroy.push(id);

    }

    fn set_renderable(&mut self, id: GameObjectId, mesh: MeshId){
        self.add_renderable(id, Mesh{mesh_id: mesh, texture_id: None});
    }

    fn set_camera_position(&mut self, new_pos: Vec3) {
        // println!("setting camera_pos to {:?}", new_pos);
        self.camera_position = new_pos;
    }
}

unsafe impl<C: Controller> Send for WorldData<C>{}
unsafe impl<C: Controller> Sync for WorldData<C>{}

impl<C: Controller> WorldData<C> {
    pub fn add_renderable(&mut self, id: GameObjectId,mesh: Mesh){
        self.renderables.insert(id, mesh);
    }
    pub fn exists(&self, id: GameObjectId) -> bool {
        self.objects.contains_key(id)
    }

    pub fn remove_marked(&mut self, scripts: &mut Data<C>) {
        let objects = std::mem::replace(&mut self.to_destroy, vec![]);
        for obj in objects.into_iter() {
            // might have been removed as child of other object
            if !self.exists(obj) {
                continue;
            }
            self.remove_game_object(obj, scripts);
        }
    }


    pub fn remove_game_object(&mut self, id: GameObjectId, scripts: &mut Data<C>) {
        let ent_id = self.other_id[id];
        let data = &self.object_data[id];
        for child in data.children.clone() {
            self.remove_game_object(child, scripts);
        }
        self.remove_single(id, scripts);
    }

    fn remove_single(&mut self, id: GameObjectId, scripts: &mut Data<C>) {

        let ent_id = self.other_id[id];
        self.wor.delete(ent_id);
        drop(ent_id);
        self.other_id.remove(id);
        self.set_parent(id, None);
        self.objects.remove(id);
        self.object_data.remove(id);
        self.renderables.remove(id);
        
        scripts.remove(id);
    }
}

use crate::math::Matrix;

impl<C: Controller> RenderingData for WorldData<C> {
    fn get_projection_matrix(&self) -> Matrix {
        let top = self.viewport_height / 2.0f32;
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
        nalgebra_glm::quat_to_mat4(&self.camera_rotation) * nalgebra_glm::translation(&(-self.camera_position))
    }

    fn get_rendering_constant(&self, name: &str) -> Value {
        match name {
            "projection_mat" => Value::Matrix4(self.get_projection_matrix()),
            "view_mat" => Value::Matrix4(self.get_view_matrix()),
            "lightColor" => Value::Vector3(Vec3::new(0.6f32, 0.6f32, 0.6f32)),
            "lightDir" => Value::Vector3(Vec3::new(2.0f32, 1.0f32, -0.1f32)),
            "camera_pos" => Value::Vector3(self.camera_position),
            _ => Value::None,
        }

    }

    fn get_renderables(
        &self,
        buffer: Option<Vec<(MeshId, Option<TextureId>, Matrix)>>
    ) -> Vec<(MeshId, Option<TextureId>, Matrix)> {
        let mut buf = match buffer {
            Some(mut vec) => {
                if vec.len() < self.renderables.len() {
                    vec.reserve(self.renderables.len() - vec.len());
                }
                vec.clear();
                vec
            }
            None => Vec::with_capacity(self.renderables.len()),
        };

        //fill here
        for renderable in &self.renderables {
            let mat = self.get_global_matrix(renderable.0);
            buf.push((renderable.1.mesh_id, renderable.1.texture_id, mat));
        }
        buf
    }
}
