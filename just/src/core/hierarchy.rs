use super::game_object::GameObject;
use super::transform::Transform;
use just_core::math::*;
use just_core::glm;
use just_core::ecs::prelude::{Entity, World};

pub struct TransformHierarchy;

impl TransformHierarchy {
    pub fn get_global_position(world: &World, id: Entity) -> Vec3 {
        let mat = Self::get_parent_matrix(world, id);
        let transform = world.get_component::<Transform>(id).unwrap();
        pos(&(mat * pos_vec(&transform.position)))
    }

    pub fn get_global_rotation(world: &World, id: Entity) -> Quat {
        let rotation = world.get_component::<Transform>(id).unwrap().rotation;
        let parent = world.get_component::<GameObject>(id).unwrap().parent;
        let parent_rotation = match parent {
            None => Quat::identity(),
            Some(parent_id) => Self::get_global_rotation(world, parent_id),
        };

        return parent_rotation * rotation;
    }

    pub fn get_global_matrix(world: &World, id: Entity) -> Matrix {
        let transform = world.get_component::<Transform>(id).unwrap();
        let mut global_mat = transform.global_matrix.borrow_mut();

        if global_mat.changed {
            let parent_matrix = Self::get_parent_matrix(world, id);
            let mat = parent_matrix * Self::get_local_matrix(world, id);

            global_mat.item = mat;
            global_mat.changed = false;
        }

        return global_mat.item;
    }

    fn get_parent_matrix(world: &World, id: Entity) -> Matrix {
        let parent = world.get_component::<GameObject>(id).unwrap().parent;
        match parent {
            None => Matrix::identity(),
            Some(parent_id) => Self::get_global_matrix(world, parent_id),
        }
    }

    fn get_local_matrix(world: &World, id: Entity) -> Matrix {
        let transform = world.get_component::<Transform>(id).unwrap();
        let mut local_matrix = transform.local_matrix.borrow_mut();

        if local_matrix.changed {
            local_matrix.item = glm::translation(&transform.position)
                * glm::quat_to_mat4(&transform.rotation)
                * glm::scaling(&transform.scale);
            local_matrix.changed = false;
        }

        return local_matrix.item;
    }

    pub fn void_local_matrix(world: &World, id: Entity) {
        let transform = world.get_component::<Transform>(id).unwrap();
        transform.local_matrix.borrow_mut().changed = true;
        drop(transform);

        Self::void_global_matrix(world, id);
    }

    fn void_global_matrix(world: &World, id: Entity) {
        let transform = world.get_component::<Transform>(id).unwrap();
        let mut global_matrix = transform.global_matrix.borrow_mut();
        if global_matrix.changed == true {
            return;
        }
        global_matrix.changed = true;

        for child in world
            .get_component::<GameObject>(id)
            .unwrap()
            .children
            .iter()
        {
            Self::void_global_matrix(world, *child);
        }
    }

    pub fn set_local_position(world: &mut World, id: Entity, new_position: Vec3) {
        let mut transform = world.get_component_mut::<Transform>(id).unwrap();
        transform.position = new_position;
        drop(transform);
        Self::void_local_matrix(world, id);
    }

    pub fn set_local_rotation(world: &mut World, id: Entity, new_rotation: Quat) {
        let mut transform = world.get_component_mut::<Transform>(id).unwrap();
        transform.rotation = new_rotation;
        drop(transform);
        Self::void_local_matrix(world, id);
    }

    pub fn set_local_scale(world: &mut World, id: Entity, new_scale: Vec3) {
        let mut transform = world.get_component_mut::<Transform>(id).unwrap();
        transform.scale = new_scale;
        drop(transform);
        Self::void_local_matrix(world, id);
    }

    pub fn get_local_position(world: &World, id: Entity) -> Vec3 {
        let transform = world.get_component::<Transform>(id).unwrap();
        return transform.position;
    }

    pub fn get_local_rotation(world: &World, id: Entity) -> Quat {
        let transform = world.get_component::<Transform>(id).unwrap();
        return transform.rotation;
    }

    pub fn get_local_scale(world: &World, id: Entity) -> Vec3 {
        let transform = world.get_component::<Transform>(id).unwrap();
        return transform.scale;
    }

    pub fn get_parent(world: &World, id: Entity) -> Option<Entity> {
        world.get_component::<GameObject>(id).unwrap().parent
    }

    pub fn set_parent(world: &mut World, id: Entity, new_parent: Option<Entity>) -> Result<(), ()> {
        if !world.is_alive(id) {
            return Result::Err(());
        }
        match new_parent {
            Some(x) => {
                if !world.is_alive(x) {
                    return Result::Err(());
                }
                world
                    .get_component_mut::<GameObject>(x)
                    .unwrap()
                    .children
                    .push(id);
            }
            None => (),
        }
        let parent = Self::get_parent(world, id);
        match parent {
            None => (),
            Some(x) => {
                let mut data = world.get_component_mut::<GameObject>(x).unwrap();
                let index = data.children.iter().position(|y| *y == id).unwrap();
                data.children.remove(index);
            }
        }
        let mut data = world.get_component_mut::<GameObject>(id).unwrap();
        data.parent = new_parent;
        drop(data);
        Self::void_local_matrix(world, id);

        Result::Ok(())
    }
}
