use just_core::glam;
use just_core::math::{Quat, Vec3};

#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,

    pub aspect_ratio: f32,
    pub fov_y: f32,
    pub z_near: f32,
    pub z_far: f32,
}

impl CameraData {
    pub fn view(&self) -> glam::Mat4 {
        glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_translation(-self.position)
    }

    pub fn projection(&self) -> glam::Mat4 {
        glam::Mat4::perspective_lh(self.fov_y, self.aspect_ratio, self.z_near, self.z_far)
    }

    pub fn view_projection(&self) -> glam::Mat4 {
        self.projection() * self.view()
    }
}
