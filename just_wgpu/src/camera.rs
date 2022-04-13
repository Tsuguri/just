use just_core::{
    math::{Vec3, Quat, Matrix},
    traits::scripting::{ScriptApiRegistry},
    ecs::prelude::*,
};


#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,
}

impl Default for CameraData {
    fn default() -> Self {
        Self {
            position: Vec3::zeros(),
            rotation: Quat::identity(),
        }
    }

}

impl CameraData {
    pub fn initialize(world: &mut World) {
        world.resources.insert::<Self>(Default::default());
    }

    pub fn view_matrix(&self) -> Matrix{
        use just_core::math::glm;
        glm::quat_to_mat4(&self.rotation) * glm::translation(&(-self.position))
    }

    pub fn cleanup(world: &mut World) {
        world.resources.remove::<Self>();
    }
}
