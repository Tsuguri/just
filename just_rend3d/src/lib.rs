mod camera;
mod obj_loader;
mod rendergraph;
mod state;

pub use camera::CameraData;

use just_core::RenderableCreationQueue;
use obj_loader::load_obj_model;
use rend3::types::{Surface, TextureFormat};
use rend3::*;
use rendergraph::RenderGraph;
use state::RendererState;
use std::sync::Arc;
use winit::dpi::{PhysicalSize, Size};
use winit::window::{Fullscreen, Window};

use image::GenericImageView;
use just_assets::{AssetManager, AssetStorage};
use just_core::ecs::prelude::*;
use just_core::ecs::world::World;
use just_core::hierarchy::TransformHierarchy;
use just_core::math::{Quat, Vec3};
pub use winit;

use winit::event_loop::EventLoop;
#[derive(Clone)]
pub struct ViewportData {
    pub camera_lens_height: f32,
    pub height: f32,
    pub width: f32,
    pub ratio: f32,
}

impl ViewportData {
    pub fn projection(&self) -> rend3::types::CameraProjection {
        rend3::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 }
    }
}
pub struct Mesh {
    pub handle: rend3::types::MeshHandle,
}
pub struct Texture {
    pub handle: rend3::types::TextureHandle,
}
pub struct RenderingSystem {}
#[derive(Default)]
pub struct Renderable {
    pub object_handle: Option<rend3::types::ObjectHandle>,
}

struct RenderingManager {
    renderer: Arc<rend3::Renderer>,
    window: winit::window::Window,
    surface: Arc<rend3::types::Surface>,
    rendergraph: RenderGraph,
    directional_handle: rend3::types::DirectionalLightHandle,
    resolution: glam::UVec2,
}

impl RenderingSystem {
    fn create_renderer(event_loop: &EventLoop<()>) -> (Arc<Renderer>, Window, Arc<Surface>, TextureFormat) {
        let window = {
            let mut builder = winit::window::WindowBuilder::new();
            builder = builder.with_title("just");

            builder
                .with_inner_size(Size::Physical(PhysicalSize::new(1920, 1080)))
                //.with_fullscreen(Some(Fullscreen::Borderless(None)))
                .build(&event_loop)
                .expect("Could not build window")
        };
        let window_size = window.inner_size();
        // Create the Instance, Adapter, and Device. We can specify preferred backend,
        // device name, or rendering profile. In this case we let rend3 choose for us.
        let iad = pollster::block_on(rend3::create_iad(None, None, None, None)).unwrap();
        // The one line of unsafe needed. We just need to guarentee that the window
        // outlives the use of the surface.
        //
        // SAFETY: this surface _must_ not be used after the `window` dies. Both the
        // event loop and the renderer are owned by the `run` closure passed to winit,
        // so rendering work will stop after the window dies.
        let surface = Arc::new(unsafe { iad.instance.create_surface(&window) });
        // Get the preferred format for the surface.
        let format = surface.get_preferred_format(&iad.adapter).unwrap();
        // Configure the surface to be ready for rendering.
        rend3::configure_surface(
            &surface,
            &iad.device,
            format,
            glam::UVec2::new(window_size.width, window_size.height),
            rend3::types::PresentMode::Mailbox,
        );
        // Make us a renderer.
        let renderer = rend3::Renderer::new(
            iad,
            rend3::types::Handedness::Left,
            Some(window_size.width as f32 / window_size.height as f32),
        )
        .unwrap();
        (renderer, window, surface, format)
    }

    pub fn initialize(world: &mut World, event_loop: &EventLoop<()>) {
        RendererState::initialize(world);
        let (renderer, window, surface, format) = Self::create_renderer(event_loop);

        // Create the base rendergraph.
        let rendergraph = RenderGraph::new(&renderer, format);

        // Set camera's location
        {
            let camera_data = world.resources.get::<CameraData>().unwrap();

            renderer.set_camera_data(rend3::types::Camera {
                projection: rend3::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
                view: camera_data.view(),
            });
        }

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        let directional_handle = renderer.add_directional_light(rend3::types::DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 10.0,
            // Direction will be normalized
            direction: glam::Vec3::new(-1.0, -4.0, 2.0),
            distance: 400.0,
        });

        let window_size = window.inner_size();
        let resolution = glam::UVec2::new(window_size.width, window_size.height);

        world.resources.insert::<RenderingManager>(RenderingManager {
            renderer,
            window,
            surface,
            rendergraph,
            resolution,
            directional_handle,
            // camera_data,
        });
        world.resources.insert::<RenderableCreationQueue>(Default::default());
    }
    pub fn maintain(_world: &mut World) {}
    pub fn update(world: &mut World) {
        let (
            manager,
            mut asset_manager,
            mut texture_storage,
            mut mesh_storage,
            camera_data,
            _viewport_data,
            mut creation_queue,
        ) = <(
            Write<RenderingManager>,
            Write<AssetManager>,
            Write<AssetStorage<Texture>>,
            Write<AssetStorage<Mesh>>,
            Read<CameraData>,
            Read<ViewportData>,
            Write<RenderableCreationQueue>,
        )>::fetch(&mut world.resources);
        texture_storage.process(&mut asset_manager, "png", |data| {
            (Self::load_png_texture(&manager, data), false)
        });

        mesh_storage.process(&mut asset_manager, "obj", |data| {
            (load_obj_model(&manager, data), false)
        });

        let to_create = std::mem::take(&mut creation_queue.queue);

        for (id, mesh, texture) in to_create.into_iter() {
            Self::add_renderable(world, id, &mesh, texture.as_deref());
        }

        // update objects transforms
        let query = <Read<Renderable>>::query();
        for (id, renderable) in query.iter_entities_immutable(world) {
            let global_matrix = TransformHierarchy::get_global_matrix(world, id);
            manager
                .renderer
                .set_object_transform(renderable.object_handle.as_ref().unwrap(), global_matrix);
        }

        manager.renderer.set_camera_data(rend3::types::Camera {
            projection: rend3::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
            view: camera_data.view(),
        });

        let frame = rend3::util::output::OutputFrame::Surface {
            surface: Arc::clone(&manager.surface),
        };
        // Ready up the renderer
        let (cmd_bufs, ready) = manager.renderer.ready();

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        manager
            .rendergraph
            .prepare_graph(&mut graph, &ready, manager.resolution);

        // Dispatch a render using the built up rendergraph!
        graph.execute(&manager.renderer, frame, cmd_bufs, &ready);
    }

    pub fn shut_down(world: &mut World) {
        RendererState::strip_down(world);
        let manago = world.resources.remove::<RenderingManager>();
        if let Some(manager) = manago {
            let RenderingManager {
                renderer,
                window,
                surface,
                rendergraph,
                ..
            } = manager;
            drop(rendergraph);
            drop(renderer);
            drop(surface);
            drop(window);
        }
    }

    /// Only to be used in scene deserialization
    /// Does not check if renderable already exists
    pub fn add_renderable(world: &mut World, id: Entity, mesh: &str, texture: Option<&str>) {
        let position = TransformHierarchy::get_global_position(world, id);
        let manager = world.resources.get::<RenderingManager>().unwrap();
        let res2 = world.resources.get::<AssetStorage<Mesh>>().unwrap();
        let res = world.resources.get::<AssetStorage<Texture>>().unwrap();
        let mesh_handle = res2.get_handle(mesh).unwrap();
        let albedo_component = match texture {
            None => rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
            Some(name) => {
                let tex_res = res.get_handle(name).unwrap();
                rend3_routine::pbr::AlbedoComponent::Texture(res.get_value(&tex_res).unwrap().handle.clone())
            }
        };
        let material = rend3_routine::pbr::PbrMaterial {
            albedo: albedo_component,
            ..rend3_routine::pbr::PbrMaterial::default()
        };
        let material_handle = manager.renderer.add_material(material);
        let object = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(res2.get_value(&mesh_handle).unwrap().handle.clone()),
            material: material_handle,
            transform: glam::Mat4::from_translation(position),
        };
        let renderable = manager.renderer.add_object(object);
        drop(manager);
        drop(res2);
        drop(res);
        world.add_component(
            id,
            Renderable {
                object_handle: Some(renderable),
            },
        );
    }
}

impl RenderingSystem {
    fn load_png_texture(renderer: &RenderingManager, data: &[u8]) -> Texture {
        let image = image::load_from_memory(data).unwrap();
        let image_rgba8 = image.to_rgba8();
        let texture = rend3::types::Texture {
            label: Option::None,
            data: image_rgba8.to_vec(),
            format: rend3::types::TextureFormat::Rgba8UnormSrgb,
            size: glam::UVec2::new(image.dimensions().0, image.dimensions().1),
            mip_count: rend3::types::MipmapCount::ONE,
            mip_source: rend3::types::MipmapSource::Uploaded,
        };
        let texture_handle = renderer.renderer.add_texture_2d(texture);
        Texture { handle: texture_handle }
    }
}
