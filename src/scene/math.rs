use crate::glm;

type Fl = f32;

pub type Vec3 = glm::TVec3<Fl>;
pub type Vec4 = glm::TVec4<Fl>;
pub type Matrix = glm::TMat4<Fl>;
pub type Quat = glm::Qua<Fl>;


pub fn pos_vec(pos: &Vec3) -> Vec4 {
    Vec4::new(pos.data[0], pos.data[1], pos.data[2], 1.0f32)
}

pub fn dir_vec(pos: &Vec3) -> Vec4 {
    Vec4::new(pos.data[0], pos.data[1], pos.data[2], 0.0f32)
}

pub fn pos(vec: &Vec4) -> Vec3 {
    glm::vec4_to_vec3(vec)
}