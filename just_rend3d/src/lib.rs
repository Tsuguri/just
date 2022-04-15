use rend3::*;
use std::sync::Arc;

use just_assets::{AssetManager, AssetStorage, Handle};
use just_core::ecs::world::World;
use just_core::math::{Quat, Vec3};
use just_core::traits::scripting::ScriptApiRegistry;
pub use winit;

use winit::event_loop::EventLoop;

#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone)]
pub struct ViewportData {
    pub camera_lens_height: f32,
    pub height: f32,
    pub width: f32,
    pub ratio: f32,
}
pub struct Mesh {}
pub struct Texture {}
pub struct RenderingSystem {}
#[derive(Default)]
pub struct Renderable {
    pub mesh_handle: Option<Handle<Mesh>>,
    pub texture_handle: Option<Handle<Texture>>,
}

struct RenderingManager {
    renderer: Arc<rend3::Renderer>,
    window: winit::window::Window,
}

impl RenderingSystem {
    pub fn initialize(world: &mut World, event_loop: &EventLoop<()>) {
        Self::initialize_state(world);
        let window = {
            let mut builder = winit::window::WindowBuilder::new();
            builder = builder.with_title("just");
            builder.build(&event_loop).expect("Could not build window")
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
        world
            .resources
            .insert::<RenderingManager>(RenderingManager { renderer, window });
    }
    pub fn maintain(world: &mut World) {}
    pub fn update(world: &mut World) {
        // render here
    }
    pub fn register_api<SAR: ScriptApiRegistry>(registry: &mut SAR) {}
    pub fn shut_down(world: &mut World) {
        let manago = world.resources.remove::<RenderingManager>();
        if let Some(manager) = manago {
            let RenderingManager { renderer, window } = manager;
            drop(renderer);
            drop(window);
        }
    }
}

impl RenderingSystem {
    fn initialize_state(world: &mut World) {
        world.resources.insert(CameraData {
            position: Vec3::zeros(),
            rotation: Quat::identity(),
        });
        world.resources.insert(ViewportData {
            width: 0.0f32,
            height: 0.0f32,
            ratio: 1.0f32,
            camera_lens_height: 10.0f32,
        });
        let asset_manager = world.resources.get::<AssetManager>().unwrap();
        let mesh_storage = AssetStorage::empty(&asset_manager, &["obj"]);
        let texture_storage = AssetStorage::empty(&asset_manager, &["png"]);
        drop(asset_manager);

        world.resources.insert::<AssetStorage<Mesh>>(mesh_storage);
        world.resources.insert::<AssetStorage<Texture>>(texture_storage);
    }
}
