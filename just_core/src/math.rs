use glam::Vec4Swizzles;

pub type Fl = f32;

pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type Matrix3 = glam::Mat3;
pub type Matrix = glam::Mat4;
pub type Quat = glam::Quat;

pub fn pos_vec(pos: &Vec3) -> Vec4 {
    pos.extend(1.0f32)
}

pub fn dir_vec(pos: &Vec3) -> Vec4 {
    pos.extend(0.0f32)
}

pub fn pos(vec: &Vec4) -> Vec3 {
    vec.xyz()
}
