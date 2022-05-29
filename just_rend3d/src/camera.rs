use glam::Mat4;
use just_core::math::{Quat, Vec3};

#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,
}

impl CameraData {
    pub fn view(&self) -> Mat4 {
        glam::Mat4::from_quat(self.rotation) * glam::Mat4::from_translation(-self.position)
    }
}
