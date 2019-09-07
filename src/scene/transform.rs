use super::WorldData;
use super::math::{Vec3, Matrix, Quat};
use crate::scene::traits::*;

impl WorldData {
    pub fn get_global_position(&self, id: GameObjectId) -> Vec3 {
        self.object_data[id].get_global_position(self)
    }

    pub fn get_global_rotation(&self, id: GameObjectId) -> Quat {
        self.object_data[id].get_global_rotation(self)
    }

    pub fn get_global_matrix(&self, id: GameObjectId) -> Matrix {
        //let obj_data = &mut self.object_data[id];
        self.object_data[id].get_global_matrix(self)
    }

    pub fn set_local_position(&mut self, id: GameObjectId, new_position: Vec3) {
        self.object_data[id].set_local_position(self, new_position);
    }

    pub fn set_local_rotation(&mut self, id: GameObjectId, new_rotation: Quat) {
        self.object_data[id].set_local_rotation(self, new_rotation);
    }

    pub fn get_local_position(&self, id: GameObjectId) -> Vec3 {
        self.object_data[id].get_local_position()
    }

    pub fn get_local_rotation(&self, id: GameObjectId) -> Quat {
        self.object_data[id].get_local_rotation()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra_glm::vec3;
    use crate::scene::MockEngine;

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

