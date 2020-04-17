use std::cell::RefCell;
use super::WorldData;
use crate::math::*;
use crate::traits::*;
use legion::prelude::*;

pub trait Ident {
    fn empty() -> Self;
}

impl Ident for Matrix {
    fn empty() -> Self {
        Matrix::identity()
    }
}

impl Ident for Quat {
    fn empty() -> Self {
        Quat::identity()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ItemState<T: Ident> {
    pub changed: bool,
    pub item: T,
}

pub type MatrixState = ItemState<Matrix>;

impl<T: Ident> ItemState<T> {
    fn new() -> Self {
        Self{
            changed: true,
            item: T::empty(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub local_matrix: RefCell<MatrixState>,
    pub global_matrix: RefCell<MatrixState>,
}

unsafe impl Send for Transform {}
unsafe impl Sync for Transform {}

impl Transform {
    pub fn new() -> Self {
        Transform {
            position: Vec3::zeros(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::identity(),
            local_matrix: RefCell::new(MatrixState::new()),
            global_matrix: RefCell::new(MatrixState::new()),
        }
    }

}
impl WorldData {
    pub fn get_global_position(&self, id: Entity) -> Vec3 {
        let mat = self.get_parent_matrix(id);
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        pos(&(mat*pos_vec(&transform.position)))
    }

    pub fn get_global_rotation(&self, id: Entity) -> Quat {
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        let rotation = transform.rotation;
        drop(transform);
        let go_id = self.wor.get_component::<GameObjectId>(id).unwrap();

        let parent_rotation = match self.object_data[*go_id].parent {
            None => Quat::identity(),
            Some(parent_id) => self.get_global_rotation(parent_id),
        };
        parent_rotation*(rotation)
    }

    pub fn get_global_matrix(&self, id: legion::prelude::Entity) -> Matrix{
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        let mut global_mat = transform.global_matrix.borrow_mut();

        if global_mat.changed {
            let parent_matrix = self.get_parent_matrix(id);
            let mat = parent_matrix * self.get_local_matrix(id);

            global_mat.item = mat;
            global_mat.changed = false;
        }


        return global_mat.item;

    }

    fn get_parent_matrix(&self, id: Entity) -> Matrix{
        let go_id = self.wor.get_component::<GameObjectId>(id).unwrap();
        match self.object_data[*go_id].parent {
            None => Matrix::identity(),
            Some(parent_id) => {
                self.get_global_matrix(parent_id)
            }
        }

    }

    fn get_local_matrix(&self, id: Entity) -> Matrix{
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        let mut local_matrix = transform.local_matrix.borrow_mut();

        if local_matrix.changed {
            local_matrix.item = crate::glm::translation(&transform.position) * crate::glm::quat_to_mat4(&transform.rotation) * crate::glm::scaling(&transform.scale);
            local_matrix.changed = false;

        }

        return local_matrix.item;
    }

    pub fn void_local_matrix(&self, id: Entity) {
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        transform.local_matrix.borrow_mut().changed = true;
        drop(transform);
        
        self.void_global_matrix(id);

    }

    fn void_global_matrix(&self, id: Entity) {
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        let mut global_matrix = transform.global_matrix.borrow_mut();
        if global_matrix.changed == true {
            return;
        }
        global_matrix.changed = true;

        let go_id = self.wor.get_component::<GameObjectId>(id).unwrap();
        for child in self.object_data[*go_id].children.clone() {
            self.void_global_matrix(child);
        }

    }

    pub fn set_local_position(&mut self, id: Entity, new_position: Vec3) {
        let mut transform = self.wor.get_component_mut::<Transform>(id).unwrap();
        transform.position = new_position;
        drop(transform);
        self.void_local_matrix(id);
    }

    pub fn set_local_rotation(&mut self, id: Entity, new_rotation: Quat) {
        let mut transform = self.wor.get_component_mut::<Transform>(id).unwrap();
        transform.rotation = new_rotation;
        drop(transform);
        self.void_local_matrix(id);
    }
    
    pub fn set_local_scale(&mut self, id: Entity, new_scale: Vec3) {
        let mut transform = self.wor.get_component_mut::<Transform>(id).unwrap();
        transform.scale = new_scale;
        drop(transform);
        self.void_local_matrix(id);
    }

    pub fn get_local_position(&self, id: Entity) -> Vec3 {
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        return transform.position;
    }

    pub fn get_local_rotation(&self, id: Entity) -> Quat {
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        return transform.rotation;
    }
    
    pub fn get_local_scale(&self, id: Entity) -> Vec3 {
        let transform = self.wor.get_component::<Transform>(id).unwrap();
        return transform.scale;
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;
    use crate::core::MockEngine;

    #[test]
    fn position() {
        let mut scene = MockEngine::mock();
        let obj = scene.create_game_object();
        let pos = vec3(0.0, 0.0, 1.0);
        scene.world.set_local_position(obj, pos);
        let pos2 = scene.world.get_global_position(obj);
        assert_eq!(pos, pos2);
    }

    #[test]
    fn position_with_parent() {
        let mut scene = MockEngine::mock();
        let obj = scene.create_game_object();
        let obj2 = scene.create_game_object();

        scene.set_parent(obj, Option::Some(obj2)).unwrap();
        let pos = vec3(0.0, 0.0, 1.0);
        let pos2 = vec3(0.0, 0.0, 2.0);
        scene.world.set_local_position(obj, pos);
        scene.world.set_local_position(obj2, pos2);

        // invalidated by parent
        assert!(!scene.world.object_data[obj].valid_global());

        let result_pos = scene.world.get_global_position(obj);
        assert_eq!(result_pos, pos + pos2);
    }

    #[test]
    fn new_parent_invalidates_global_matrix() {
        let mut scene = MockEngine::mock();
        let obj = scene.create_game_object();
        let obj2 = scene.create_game_object();
        let obj3 = scene.create_game_object();

        scene.set_parent(obj, Some(obj2)).unwrap();
        assert!(!scene.world.object_data[obj].valid_global());
        scene.world.get_global_matrix(obj);
        assert!(scene.world.object_data[obj].valid_global());
        scene.set_parent(obj, Some(obj3)).unwrap();
        assert!(!scene.world.object_data[obj].valid_global());
    }

    #[test]
    fn parent_rotation_applied() {
        let mut scene = MockEngine::mock();
        let obj = scene.create_game_object();
        let obj2 = scene.create_game_object();

        scene.set_parent(obj, Some(obj2)).unwrap();
        let rot1 = nalgebra_glm::quat_angle_axis(nalgebra_glm::half_pi(), &Vec3::new(0.0, 0.0, 1.0));
        let rot2 = nalgebra_glm::quat_angle_axis(nalgebra_glm::half_pi(), &Vec3::new(0.0, 0.0, -1.0));

        scene.world.set_local_rotation(obj, rot1);
        scene.world.set_local_rotation(obj2, rot2);

        let result_rot = scene.world.get_global_rotation(obj);
        assert!(similar(&result_rot, &Quat::identity()));
    }


    fn similar(quat1: &Quat, quat2: &Quat) -> bool {
        nalgebra_glm::quat_equal_eps(quat1, quat2, 0.0001f32) == nalgebra_glm::TVec4::new(true, true, true, true)
    }
}

