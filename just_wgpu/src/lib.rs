use just_assets::{AssetManager, AssetStorage, Handle};
use just_core::{
    ecs::prelude::*,
    math::{Vec3, Quat},
};

use wgpu::util::DeviceExt;
use just_core::hierarchy::TransformHierarchy;

pub use winit;
use winit::{
    event_loop::{EventLoop},
    window::{Window, WindowBuilder},
};

use futures::executor::block_on;

mod texture;
mod viewport;
mod camera;
mod renderable;
mod mesh;
mod ubershader;

pub use camera::CameraData;
use ubershader::Ubershader;


#[derive(Clone)]
pub struct ViewportData {
    pub camera_lens_height: f32,
    pub height: f32,
    pub width: f32,
    pub ratio: f32,
}

pub struct Mesh {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_count: u32,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub bind_group: Option<wgpu::BindGroup>,
}

pub struct Renderable {
    pub mesh_handle: Option<Handle<Mesh>>,
    pub texture_handle: Option<Handle<Texture>>,
}

impl Default for Renderable {
    fn default() -> Self {
        Self {
            mesh_handle: None,
            texture_handle: None,
        }
    }

}

pub struct RenderingSystem;

struct Hardware {
    window: Window,
}

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swapchain_desc: wgpu::SwapChainDescriptor,
    swapchain: wgpu::SwapChain,
    size: winit::dpi::PhysicalSize<u32>,

    mesh: Mesh,

    diffuse_texture: Texture,
    depth_texture: Texture,
    ubershader: Ubershader,
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32;3],
    normal: [f32;3],
    tex_coords: [f32;2],

}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float3,
                },
                wgpu::VertexAttributeDescriptor {
                    offset: std::mem::size_of::<[f32;3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float3,

                },
                wgpu::VertexAttributeDescriptor {
                    offset: (std::mem::size_of::<[f32;3]>() * 2) as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,

                },
            ],
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

fn create_depth_texture(device: &wgpu::Device, size: wgpu::Extent3d, label: Option<&str>) -> Texture {
    let desc = wgpu::TextureDescriptor {
        label,
        dimension: wgpu::TextureDimension::D2,
        size,
        mip_level_count: 1,
        sample_count: 1,
        usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        format: wgpu::TextureFormat::Depth32Float,
    };

    let texture = device.create_texture(&desc);
    let view = texture.create_view(&Default::default());

    Texture{
        texture,
        view,
        bind_group: None,
    }
}

impl State {
    async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::BackendBit::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS,
                limits: wgpu::Limits {
                    max_push_constant_size: 256,
                    ..Default::default()
                },
                shader_validation: true,
            },
            None
        ).await.unwrap();

        let swapchain_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swapchain = device.create_swap_chain(&surface, &swapchain_desc);

        let ubershader = Ubershader::compile(&device, swapchain_desc.format);

        let texture_data = include_bytes!("../../dev_app/res/tex1.png");
        let diffuse_texture = RenderingSystem::load_texture(&device, &queue, (&ubershader.texture_bind_group_layout, &ubershader.sampler), texture_data);


        let depth_texture = create_depth_texture(&device, wgpu::Extent3d {
            height: size.height,
            width: size.width,
            depth: 1,
        }, Some("main depth texture"));


        let data = include_bytes!("../../dev_app/res/monkey.obj");
        let mesh = RenderingSystem::load_model(&device, data);


        Self {
            surface,
            device,
            queue,
            swapchain_desc,
            swapchain,
            size,
            mesh,
            diffuse_texture,
            depth_texture,
            ubershader,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.swapchain_desc.height = new_size.height;
        self.swapchain_desc.width = new_size.width;
        self.swapchain = self.device.create_swap_chain(&self.surface, &self.swapchain_desc);
        self.depth_texture = create_depth_texture(&self.device, wgpu::Extent3d {
            height:new_size.height,
            width:new_size.width,
            depth:1,
        }, Some("main depth texture"));
    }
}

impl RenderingSystem {
    pub fn initialize(world: &mut World, event_loop: &EventLoop<()>) {
        let window = WindowBuilder::new().with_title("It's Just Game").build(event_loop).unwrap();

        CameraData::initialize(world);
        
        let state = block_on(State::new(&window));
        let size = state.size;
        world.resources.insert(ViewportData {
            width: size.width as f32,
            height: size.height as f32,
            ratio: (size.width / size.height) as f32,
            camera_lens_height: 10.0f32,
        });

        world.resources.insert::<Hardware>(Hardware{window});
        world.resources.insert::<State>(state);
        let asset_manager = world.resources.get::<AssetManager>().unwrap();
        let mesh_storage = AssetStorage::empty(&asset_manager, &["obj"]);
        let texture_storage = AssetStorage::empty(&asset_manager, &["png"]);
        drop(asset_manager);
        world.resources.insert::<AssetStorage<Mesh>>(mesh_storage);
        world.resources.insert::<AssetStorage<Texture>>(texture_storage);

    }

    pub fn maintain(world: &mut World) {

    }

    fn load_texture(device: &wgpu::Device, queue: &wgpu::Queue, bind_layout_sampler: (&wgpu::BindGroupLayout, &wgpu::Sampler), data: &[u8]) -> Texture {
        let image = image::load_from_memory(data).unwrap();

        use image::GenericImageView;
        let dimensions = image.dimensions();
        let rgba = image.into_rgba();


        let im_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor{
            size: im_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
            label: Some("some tex1 diff texture"),
        });

        queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: 4 * dimensions.0,
                rows_per_image: dimensions.1,
            },
            im_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &bind_layout_sampler.0,
            label: Some("diffuse bind group"),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&bind_layout_sampler.1),
                }
            ],
        });

        Texture {
            texture,
            view,
            bind_group: Some(bind_group),
        }

    }

    fn load_model(device: &wgpu::Device, data: &[u8]) -> Mesh {
        let mut cursor = std::io::Cursor::new(data);
        let (mut objects, materials) = tobj::load_obj_buf(&mut cursor, true, |arg|{
            Result::Err(tobj::LoadError::ReadError)
        }).unwrap();

        let first_mesh = objects.pop().unwrap();
        println!("processing mesh {}", first_mesh.name);
        let mut first_mesh = first_mesh.mesh;

        let num_vertices = first_mesh.positions.len() / 3;
        let mut vertices = Vec::with_capacity(num_vertices);

        if first_mesh.normals.len() == 0 {
            first_mesh.normals.resize(num_vertices * 3, 0.5);
            println!("setting normals with {}, results in {} normals", num_vertices*3, first_mesh.normals.len());
        }
        if first_mesh.texcoords.len() == 0 {
            first_mesh.texcoords.resize(num_vertices*2, 0.5);
            println!("setting uvs with {}, results in {} uvs", num_vertices*2, first_mesh.texcoords.len());
        }

        for i in 0..first_mesh.positions.len() / 3 {
            vertices.push(Vertex{
                position: [first_mesh.positions[i * 3], first_mesh.positions[i * 3 + 1], first_mesh.positions[i * 3 + 2]],
                normal: [first_mesh.normals[i*3], first_mesh.normals[i*3 + 1], first_mesh.normals[i*3 + 2]],
                tex_coords: [first_mesh.texcoords[i*2], first_mesh.texcoords[i*2 + 1]],
            });
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: Some("model vertices"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_count = first_mesh.indices.len() as u32;

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("model indices"),
            usage: wgpu::BufferUsage::INDEX,
            contents: bytemuck::cast_slice(&first_mesh.indices),
        });



        Mesh {
            vertex_buffer,
            index_buffer,
            index_count,
        }
    }
    fn projection_matrix(viewport_data: &ViewportData) -> just_core::math::Matrix {

        let top = viewport_data.camera_lens_height / 2.0f32;
        let bot = -top;
        let right = viewport_data.ratio * top;
        let left = -right;
        let near = -50.0f32;
        let far = 300.0f32;
        let mut temp = just_core::math::glm::ortho_lh_zo(left, right, bot, top, near, far);
        // let mut temp = glm::perspective_lh_zo(
        //     256.0f32 / 108.0, f32::to_radians(45.0f32), 0.1f32, 100.0f32);
        //temp[(1, 1)] *= -1.0;
        temp
    }

    pub fn update(world: &mut World) {

        let (
            mut state,
            mut asset_manager,
            mut texture_storage,
            mut mesh_storage,
            camera_data,
            viewport_data) = <(
                Write::<State>,
                Write::<AssetManager>, 
                Write::<AssetStorage<Texture>>,
                Write::<AssetStorage<Mesh>>,
                Read::<CameraData>,
                Read::<ViewportData>)>::fetch(&mut world.resources);

        texture_storage.process(&mut asset_manager, "png", |data| {
            (Self::load_texture(&state.device, &state.queue, (&state.ubershader.texture_bind_group_layout, &state.ubershader.sampler), data), false)
        });

        mesh_storage.process(&mut asset_manager, "obj", |data| {
            (Self::load_model(&state.device, data), false)
        });
        let view_matrix = camera_data.view_matrix();
        
        let projection_matrix = Self::projection_matrix(&viewport_data);

        let query = <Read<Renderable>>::query();



        let frame = state.swapchain.get_current_frame().expect("Timeout getting frame texture").output;

        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder"),
        });
        // main color renderpass
        {
            let mut renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                color_attachments: &[
                    wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &frame.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: true,
                        }

                    }
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor{
                    attachment: &state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,

                }),
            });

            renderpass.set_pipeline(&state.ubershader.render_pipeline);

            // per view matrices
            let view_offset: u32 = 0;
            let projection_offset: u32 = 16 * 4;
            renderpass.set_push_constants(wgpu::ShaderStage::VERTEX, view_offset, bytemuck::cast_slice(&view_matrix.data));
            renderpass.set_push_constants(wgpu::ShaderStage::VERTEX, projection_offset, bytemuck::cast_slice(&projection_matrix.data));

            let model_offset: u32 = 16 * 4 * 2;

            for (entity_id, mesh) in query.iter_entities_immutable(world) {
                let mat = TransformHierarchy::get_global_matrix(world, entity_id);
                match mesh.mesh_handle {
                    None =>{
                        println!("\trenderable with empty mesh");
                    },
                    Some(x) => {

                        renderpass.set_push_constants(wgpu::ShaderStage::VERTEX, model_offset, bytemuck::cast_slice(&mat.data));
                        let model = mesh_storage.get_value(&x).unwrap();

                        renderpass.set_vertex_buffer(0, model.vertex_buffer.slice(..));
                        renderpass.set_index_buffer(model.index_buffer.slice(..));

                        match mesh.texture_handle {
                            None => {
                                // bind default texture
                                let bind_group = (&state.diffuse_texture.bind_group).as_ref().unwrap();
                                renderpass.set_bind_group(0, &bind_group, &[]);
                            }
                            Some(tex) => {
                                let texture = texture_storage.get_value(&tex).unwrap();
                                let bind_group = (&texture.bind_group).as_ref().unwrap();
                                renderpass.set_bind_group(0, &bind_group, &[]);
                            }
                        }

                        renderpass.draw_indexed(0..model.index_count, 0, 0..1);
                    }
                }
            }



        }
        state.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn shut_down(world: &mut World) {
        world.resources.remove::<AssetStorage<Mesh>>();
        world.resources.remove::<AssetStorage<Texture>>();
        world.resources.remove::<State>();
        world.resources.remove::<Hardware>();

    }

}
