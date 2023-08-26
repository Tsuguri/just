mod camera;
mod model;
mod obj_loader;
mod postprocessing;
mod screen_data;
mod standard_pass;
mod state;
mod texture;
mod ui;
mod vertex;
mod viewport;

mod tile_renderer;

use std::collections::HashMap;
use std::ops::{Deref, DerefMut, Range};

pub use camera::CameraData;
use camera::CameraUniform;
use egui::RawInput;
use egui_wgpu::renderer::ScreenDescriptor;
use just_core::hierarchy::TransformHierarchy;
use model::{MeshData, MeshVertex};
pub use screen_data::ScreenData;
use viewport::ViewportData;

use just_assets::{AssetManager, AssetStorage};
use just_core::ecs::prelude::*;
use just_core::ecs::world::World;
use just_core::RenderableCreationQueue;
use just_input::{InputChannel, KeyboardState, MouseState};
use obj_loader::load_obj_model;
use postprocessing::PostprocessingPass;
use standard_pass::StandardPass;
use state::RendererState;
pub use ui::Ui;
use wgpu::Extent3d;

use tile_renderer::TileRenderer;
pub use winit;

use texture::TextureData;

use just_core::glam;

use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Mesh(u32);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct Texture(u32);

pub struct RenderingSystem {}

#[derive(Default)]
pub struct Renderable {
    mesh: Mesh,
    texture: Texture,
}

struct EguiSystem {
    renderer: egui_wgpu::Renderer,
}

impl EguiSystem {
    pub fn initialize(world: &mut World, device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let renderer = egui_wgpu::renderer::Renderer::new(device, format, None, 1);
        Ui::initialize(world);
        EguiSystem { renderer }
    }
}

struct RenderingManager {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: winit::window::Window,
    depth_texture: TextureData,
    middle_render_target: TextureData,
    mrt_bind_group: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    camera_uniform: CameraUniform,
    meshes: HashMap<Mesh, MeshData>,
    textures: HashMap<Texture, TextureData>,
    texture_bindings: HashMap<Texture, wgpu::BindGroup>,
    tile_renderer: TileRenderer,
    standard_pass: StandardPass,
    postprocessing_pass: PostprocessingPass,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ModelUniform {
    model: [[f32; 4]; 4],
}

impl ModelUniform {
    fn new() -> Self {
        Self {
            model: glam::Mat4::IDENTITY.to_cols_array_2d(),
        }
    }

    fn update_matrix(&mut self, model_matrix: glam::Mat4) {
        self.model = model_matrix.to_cols_array_2d();
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a MeshData);
    fn draw_mesh_instanced(&mut self, mesh: &'a MeshData, instances: Range<u32>);
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'a MeshData) {
        self.draw_mesh_instanced(mesh, 0..1);
    }
    fn draw_mesh_instanced(&mut self, mesh: &'a MeshData, instances: Range<u32>) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
}

impl RenderingSystem {
    async fn initialize_wgpu(event_loop: &EventLoop<()>, world: &mut World) -> RenderingManager {
        let screen_data = world.resources.get::<ScreenData>().unwrap();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::<u32>::new(1920, 1080))
            .build(&event_loop)
            .unwrap();

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::PUSH_CONSTANTS,
                    limits: wgpu::Limits {
                        max_push_constant_size: 64,
                        ..Default::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_capabilities = surface.get_capabilities(&adapter);

        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture = TextureData::create_depth_texture(&device, &config);

        let middle_render_target = TextureData::create_render_target(
            &device,
            Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            config.format,
            "middle target",
        );

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });
        let mrt_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&middle_render_target.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&middle_render_target.sampler),
                },
            ],
            label: Some("mrt_bind_group"),
        });

        let mut camera_uniform = CameraUniform::new_uniform(&device);
        camera_uniform.update_view_projection(&screen_data.camera);
        drop(screen_data);

        let tiles = TileRenderer::new();
        let standard_pass =
            StandardPass::initialize(&device, config.format, &camera_uniform, &texture_bind_group_layout);
        let postprocessing_pass = PostprocessingPass::initialize(&device, config.format, &texture_bind_group_layout);

        let egui = EguiSystem::initialize(world, &device, config.format);
        world.resources.insert(egui);

        RenderingManager {
            surface,
            device,
            queue,
            config,
            size,
            window,
            depth_texture,
            middle_render_target,
            mrt_bind_group,
            texture_bind_group_layout,
            camera_uniform,
            meshes: Default::default(),
            textures: Default::default(),
            texture_bindings: Default::default(),
            tile_renderer: tiles,
            standard_pass,
            postprocessing_pass,
        }
    }

    pub fn resize(world: &mut World, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            {
                let mut screen_data = world.resources.get_mut::<ScreenData>().unwrap();
                screen_data.viewport.viewport_resized(&new_size);
            }
            let mut manager = world.resources.get_mut::<RenderingManager>().unwrap();
            manager.size = new_size;
            manager.config.width = new_size.width;
            manager.config.height = new_size.height;
            manager.surface.configure(&manager.device, &manager.config);
            manager.depth_texture = TextureData::create_depth_texture(&manager.device, &manager.config);
            manager.middle_render_target = TextureData::create_render_target(
                &manager.device,
                Extent3d {
                    width: manager.config.width,
                    height: manager.config.height,
                    depth_or_array_layers: 1,
                },
                manager.config.format,
                "middle RT",
            );
            manager.mrt_bind_group = manager.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &manager.texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&manager.middle_render_target.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&manager.middle_render_target.sampler),
                    },
                ],
                label: Some("mrt_bind_group"),
            });
        }
    }

    pub fn initialize(world: &mut World, event_loop: &EventLoop<()>) {
        RendererState::initialize(world);
        env_logger::init();

        let manager = pollster::block_on(Self::initialize_wgpu(event_loop, world));

        world.resources.insert::<RenderingManager>(manager);
        world.resources.insert::<RenderableCreationQueue>(Default::default());
    }

    pub fn maintain(_world: &mut World) {}
    pub fn update(world: &mut World) {
        let (
            mut manager,
            mut egui,
            mut asset_manager,
            mut texture_storage,
            mut mesh_storage,
            mut ui,
            keyboard,
            screen_data,
            mut creation_queue,
        ) = <(
            Write<RenderingManager>,
            Write<EguiSystem>,
            Write<AssetManager>,
            Write<AssetStorage<Texture>>,
            Write<AssetStorage<Mesh>>,
            Write<Ui>,
            Read<KeyboardState>,
            Read<ScreenData>,
            Write<RenderableCreationQueue>,
        )>::fetch(&mut world.resources);

        // loading requested assets
        texture_storage.process(&mut asset_manager, "png", |data, name| {
            (Self::load_png_texture(&mut manager, data, name), false)
        });

        mesh_storage.process(&mut asset_manager, "obj", |data, name| {
            (load_obj_model(&mut manager, data, name), false)
        });

        //creating renderables requested by game logic
        let to_create = std::mem::take(&mut creation_queue.queue);

        for (id, mesh, texture) in to_create.into_iter() {
            Self::add_renderable(world, id, &mesh, &texture);
        }

        // update camera data
        manager.camera_uniform.update_view_projection(&screen_data.camera);
        manager.queue.write_buffer(
            &manager.camera_uniform.buffer,
            0,
            bytemuck::cast_slice(&[manager.camera_uniform.view_projection]),
        );

        let egui_output = Ui::update(world);
        let output = manager.surface.get_current_texture().unwrap();

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = manager.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let mut model_uniform = ModelUniform::new();

        // render stuff
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &manager.middle_render_target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &manager.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(&manager.standard_pass.render_pipeline);
            render_pass.set_bind_group(1, &manager.camera_uniform.bind_group, &[]);

            let query = <Read<Renderable>>::query();
            for (id, renderable) in query.iter_entities_immutable(world) {
                let global_matrix = TransformHierarchy::get_global_matrix(world, id);
                let mesh = renderable.mesh;
                let tex = renderable.texture;
                let tex_bind_group = manager.texture_bindings.get(&tex).unwrap();
                model_uniform.update_matrix(global_matrix);
                render_pass.set_bind_group(0, tex_bind_group, &[]);
                render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, bytemuck::cast_slice(&[model_uniform]));
                render_pass.draw_mesh_instanced(&manager.meshes.get(&mesh).unwrap(), 0..1);
            }
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pp rp"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.9,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&manager.postprocessing_pass.render_pipeline);
            render_pass.set_bind_group(0, &manager.mrt_bind_group, &[]);
            render_pass.draw(0..3, 0..1)
        }

        let device = &manager.device;
        let queue = &manager.queue;
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [manager.config.width, manager.config.height],
            pixels_per_point: 1.0f32,
        };
        let paint_jobs = ui.tessellate(egui_output.shapes);
        for update in egui_output.textures_delta.set {
            egui.renderer.update_texture(device, queue, update.0, &update.1);
        }
        for free in egui_output.textures_delta.free {
            egui.renderer.free_texture(&free);
        }

        egui.renderer
            .update_buffers(device, queue, &mut encoder, &paint_jobs, &screen_descriptor);
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui rp"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            egui.renderer.render(&mut render_pass, &paint_jobs, &screen_descriptor);
        }

        manager.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn shut_down(world: &mut World) {
        RendererState::strip_down(world);
    }

    /// Only to be used in scene deserialization
    /// Does not check if renderable already exists
    pub fn add_renderable(world: &mut World, id: Entity, mesh: &str, texture: &str) {
        let mesh_storage = world.resources.get::<AssetStorage<Mesh>>().unwrap();
        let texture_storage = world.resources.get::<AssetStorage<Texture>>().unwrap();

        let mesh_handle = mesh_storage.get_handle(mesh).unwrap();
        let mesh = *mesh_storage.get_value(&mesh_handle).unwrap();

        let texture_handle = texture_storage.get_handle(texture).unwrap();
        let texture = *texture_storage.get_value(&texture_handle).unwrap();
        drop(mesh_storage);
        drop(texture_storage);

        world.add_component(
            id,
            Renderable {
                mesh: mesh,
                texture: texture,
            },
        );
    }
}

impl RenderingSystem {
    fn load_png_texture(renderer: &mut RenderingManager, data: &[u8], name: &str) -> Texture {
        let image_data = TextureData::from_bytes(&renderer.device, &renderer.queue, data, name).unwrap();

        let last_key = renderer.textures.keys().map(|i| i.0).max().unwrap_or(0);
        let new_key = last_key + 1;
        let key = Texture(new_key);
        let texture_bind_group = renderer.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &renderer.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&image_data.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&image_data.sampler),
                },
            ],
            label: Some(&format!("{} texture bind group", name)),
        });
        renderer.texture_bindings.insert(key, texture_bind_group);
        renderer.textures.insert(key, image_data);

        key
    }
}
