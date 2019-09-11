use crate::traits::{Data, Map, MeshId, TextureId, World, GameObjectId, RenderingData, Controller};

use crate::math::*;

use super::game_object::GameObject;

pub struct Mesh {
    pub mesh_id: MeshId,
    pub texture_id: Option<TextureId>,
}

pub struct WorldData {
    // at the same time indicates if object is active
    pub objects: Map<bool>,
    pub object_data: Data<GameObject>,
    pub renderables: Data<Mesh>,
    pub to_destroy: Vec<GameObjectId>,
}

impl World for WorldData {
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
        self.object_data[obj].void_local_matrix(&self);

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
        self.object_data.insert(id, go);
        id
    }

    fn destroy_gameobject(&mut self, id: GameObjectId) {
        self.to_destroy.push(id);

    }

    fn set_renderable(&mut self, id: GameObjectId, mesh: MeshId){
        self.add_renderable(id, Mesh{mesh_id: mesh, texture_id: None});
    }
}

unsafe impl Send for WorldData{}
unsafe impl Sync for WorldData{}

impl WorldData {
    pub fn add_renderable(&mut self, id: GameObjectId,mesh: Mesh){
        self.renderables.insert(id, mesh);
    }
    pub fn exists(&self, id: GameObjectId) -> bool {
        self.objects.contains_key(id)
    }

    pub fn remove_marked<C: Controller>(&mut self, scripts: &mut Data<C>) {
        let objects = std::mem::replace(&mut self.to_destroy, vec![]);
        for obj in objects.into_iter() {
            // might have been removed as child of other object
            if !self.exists(obj) {
                continue;
            }
            self.remove_game_object(obj, scripts);
        }
    }


    pub fn remove_game_object<C: Controller>(&mut self, id: GameObjectId, scripts: &mut Data<C>) {
        let data = &self.object_data[id];
        for child in data.children.clone() {
            self.remove_game_object(child, scripts);
        }
        self.remove_single(id, scripts);
    }

    fn remove_single<C: Controller>(&mut self, id: GameObjectId, scripts: &mut Data<C>) {

        self.set_parent(id, None);
        self.objects.remove(id);
        self.object_data.remove(id);
        self.renderables.remove(id);
        scripts.remove(id);
    }
}

use crate::math::Matrix;

impl RenderingData for WorldData {
    fn get_projection_matrix(&self) -> Matrix {
        let mut temp = nalgebra_glm::perspective_lh_zo(
            256.0f32 / 108.0, f32::to_radians(45.0f32), 0.1f32, 100.0f32);
        temp[(1, 1)] *= -1.0;
        temp
    }

    fn get_view_matrix(&self) -> Matrix {
        nalgebra_glm::translation(&nalgebra_glm::vec3(1.0f32, -2.5, 10.0))
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

            let mat = self.object_data[renderable.0].get_global_matrix(self);

            buf.push((renderable.1.mesh_id, renderable.1.texture_id, mat));
        }
        buf
    }
}
