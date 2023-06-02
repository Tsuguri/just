use just_core::glam;
use just_core::math::{Quat, Vec3};
use wgpu::util::DeviceExt;

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

pub struct CameraUniform {
    pub view_projection: [[f32; 4]; 4],
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraUniform {
    pub fn new_uniform(device: &wgpu::Device) -> Self {
        let view_projection = glam::Mat4::IDENTITY.to_cols_array_2d();
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: std::mem::size_of::<CameraUniform>() as wgpu::BufferAddress,
            label: Some("Camera buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera bind group layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Camera bind group"),
        });
        Self {
            view_projection,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update_view_projection(&mut self, camera: &CameraData) {
        self.view_projection = camera.view_projection().to_cols_array_2d();
    }
}
