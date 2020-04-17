use crate::traits::{Data, Map, MeshId, TextureId, World, GameObjectId, RenderingData, Controller, Value};

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
    pub objects: Map<bool>,
    pub object_data: Data<GameObject>,
    pub to_destroy: Vec<Entity>,
    pub other_id: Data<legion::prelude::Entity>,
    pub camera_position: Vec3,
    pub camera_rotation: Quat,
    pub viewport_height: f32,

    pub wor: legion::prelude::World,
    pub res: legion::prelude::Resources,
}

#[derive(Clone)]
pub struct ObjectsToDelete(Vec<Entity>);

impl WorldData {
    pub fn new() -> WorldData {
        let mut resources = legion::prelude::Resources::default();
        resources.insert(ObjectsToDelete(Vec::new()));
        WorldData {
            objects: Map::with_key(),
            object_data: Data::new(),
            other_id: Data::new(),
            to_destroy: vec![],
            camera_position: Vec3::zeros(),
            camera_rotation: Quat::identity(),
            viewport_height: 10.0f32,
            wor: legion::prelude::World::new(),
            res: resources,
        }
    }
}

impl World for WorldData {
    fn get_legion(&mut self) -> &mut legion::prelude::World{
        &mut self.wor
    }

    fn map_id(&self, id: GameObjectId) -> legion::prelude::Entity {
        self.other_id[id]
    }

    fn get_name(&self, id: Entity) -> String{
        let go_id = self.wor.get_component::<GameObjectId>(id).unwrap();
        self.object_data[*go_id].name.clone()

    }

    fn set_name(&mut self, id: Entity, name: String){
        let go_id = self.wor.get_component::<GameObjectId>(id).unwrap();
        self.object_data[*go_id].name = name;
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
        let go_id =self.wor.get_component::<GameObjectId>(id).unwrap();
        self.object_data[*go_id].parent
    }

    fn set_parent(&mut self, obj: Entity, new_parent: Option<Entity>) -> Result<(),()>{
        let go_id =self.wor.get_component::<GameObjectId>(obj).unwrap();
        if !self.exists(*go_id) {
            return Result::Err(());
        }
        match new_parent {
            Some(x) => {
                let parent_go_id =self.wor.get_component::<GameObjectId>(x).unwrap();
                if !self.exists(*parent_go_id) {
                    return Result::Err(());
                }
                self.object_data[*parent_go_id].children.push(obj);
            }
            None => (),
        }
        match self.object_data[*go_id].parent {
            None => (),
            Some(x) => {
                let parent_go_id =self.wor.get_component::<GameObjectId>(x).unwrap();
                let index = self.object_data[*parent_go_id].children.iter().position(|y| *y == obj).unwrap();
                self.object_data[*parent_go_id].children.remove(index);
            }
        }
        self.object_data[*go_id].parent = new_parent;
        self.void_local_matrix(obj);

        Result::Ok(())
    }

    fn find_by_name(&self, name: &str) -> Vec<Entity>{
        self.object_data.iter().filter(|(x, y)| {
            y.name == name
        }).map(|(x,y)| self.map_id(x)).collect()
    }

    fn create_gameobject(&mut self) -> Entity {
        let id = self.objects.insert(true);
        let go = GameObject::new(id);

        let tr = Transform::new();
        self.object_data.insert(id, go);
        let ent_id = self.wor.insert(
            (),
            vec![
                (Transform::new(),id,),
            ],
        ).to_vec();
        self.other_id.insert(id, ent_id[0]);
        ent_id[0]
    }

    fn destroy_gameobject(&mut self, id: Entity) {
        self.to_destroy.push(id);

    }

    fn set_renderable(&mut self, id: Entity, mesh: MeshId){
        self.add_renderable(id, Mesh{mesh_id: mesh, texture_id: None});
    }

    fn set_camera_position(&mut self, new_pos: Vec3) {
        // println!("setting camera_pos to {:?}", new_pos);
        self.camera_position = new_pos;
    }
}

unsafe impl Send for WorldData{}
unsafe impl Sync for WorldData{}

impl WorldData {
    pub fn add_renderable(&mut self, id: Entity,mesh: Mesh){
        self.wor.add_component(id, mesh);
    }
    pub fn exists(&self, id: GameObjectId) -> bool {
        self.objects.contains_key(id)
    }

    pub fn remove_marked(&mut self) {
        let objects = std::mem::replace(&mut self.to_destroy, vec![]);
        for obj in objects.into_iter() {
            // might have been removed as child of other object
            let go_id = *self.wor.get_component::<GameObjectId>(obj).unwrap();
            if !self.exists(go_id) {
                continue;
            }
            self.remove_game_object(obj);
        }
    }


    pub fn remove_game_object(&mut self, id: Entity) {
        let go_id = *self.wor.get_component::<GameObjectId>(id).unwrap();
        let data = &self.object_data[go_id];
        for child in data.children.clone() {
            self.remove_game_object(child);
        }
        self.remove_single(id);
    }

    fn remove_single(&mut self, id: Entity) {
        let go_id = *self.wor.get_component::<GameObjectId>(id).unwrap();
        self.wor.delete(id);
        self.other_id.remove(go_id);
        self.set_parent(id, None);
        self.objects.remove(go_id);
        self.object_data.remove(go_id);
    }
}

use crate::math::Matrix;

impl RenderingData for WorldData {
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
