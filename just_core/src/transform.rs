use crate::math::*;
use std::cell::RefCell;

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
        Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::MockEngine;
    use nalgebra_glm::vec3;

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
        scene.world.get_global_matrix(obj);
        scene.set_parent(obj, Some(obj3)).unwrap();
    }

    #[test]
    fn parent_rotation_applied() {
        let mut scene = MockEngine::mock();
        let obj = scene.create_game_object();
        let obj2 = scene.create_game_object();

        scene.set_parent(obj, Some(obj2)).unwrap();
        let rot1 =
            nalgebra_glm::quat_angle_axis(nalgebra_glm::half_pi(), &Vec3::new(0.0, 0.0, 1.0));
        let rot2 =
            nalgebra_glm::quat_angle_axis(nalgebra_glm::half_pi(), &Vec3::new(0.0, 0.0, -1.0));

        scene.world.set_local_rotation(obj, rot1);
        scene.world.set_local_rotation(obj2, rot2);

        let result_rot = scene.world.get_global_rotation(obj);
        assert!(similar(&result_rot, &Quat::identity()));
    }

    fn similar(quat1: &Quat, quat2: &Quat) -> bool {
        nalgebra_glm::quat_equal_eps(quat1, quat2, 0.0001f32)
            == nalgebra_glm::TVec4::new(true, true, true, true)
    }
}
