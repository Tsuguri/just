pub use rendy;
pub use octo_runtime;
use rendy::{
    command::Families,
    factory::{Config, Factory},
    graph::{present::PresentNode, render::*, GraphBuilder},
    hal,
    wsi::winit::{EventsLoop, Window, WindowBuilder},
};
use std::mem::ManuallyDrop;
use just_core::ecs::prelude::*;
use just_core::math::{Vec3, Quat};

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

pub struct Hardware<B: hal::Backend> {
    pub window: Window,
    pub event_loop: EventsLoop,
    pub factory: ManuallyDrop<Factory<B>>,
    pub families: ManuallyDrop<Families<B>>,
    pub surface: Option<rendy::wsi::Surface<B>>,
    pub used_family: rendy::command::FamilyId,
}

impl<B: hal::Backend> std::ops::Drop for Hardware<B> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.families);
            ManuallyDrop::drop(&mut self.factory);
        }
    }
}

impl<B: hal::Backend> Hardware<B> {
    pub fn create() -> Self {
        let conf: rendy::factory::Config = Default::default();
        Self::new(conf)
    }
    pub fn new(config: Config) -> Self {
        let (mut factory, families): (Factory<B>, _) = rendy::factory::init(config).unwrap();
        let mut event_loop = EventsLoop::new();
        event_loop.poll_events(|_| ());

        let monitor_id = event_loop.get_primary_monitor();

        let window = WindowBuilder::new()
            .with_title("It's Just Game")
            .with_fullscreen(Some(monitor_id))
            .build(&event_loop)
            .unwrap();
        let surface = factory.create_surface(&window);
        let family_id = families
            .as_slice()
            .iter()
            .find(|family| factory.surface_support(family.id(), &surface))
            .map(rendy::command::Family::id)
            .unwrap();

        Self {
            factory: ManuallyDrop::new(factory),
            families: ManuallyDrop::new(families),
            window,
            event_loop,
            surface: Option::Some(surface),
            used_family: family_id,
        }
    }
}

pub mod resources;
pub mod deferred_node;
pub mod octo_node;
pub mod node_prelude;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Renderable {
    pub mesh: Option<resources::MeshId>,
    pub texture: Option<resources::TextureId>,
}

impl std::default::Default for Renderable {
    fn default() -> Self {
        Renderable {
            mesh: None,
            texture: None,
        }
    }
}

//#[derive(Copy, Clone, Debug, PartialEq)]
//pub struct Mesh {
    //pub mesh_id: MeshId,
    //pub texture_id: Option<TextureId>,
//}

impl Renderable {
    pub fn add_renderable_to_go(world: &mut World, id: Entity, mesh: resources::MeshId) {
        world.add_component(
            id,
            Renderable {
                mesh: Some(mesh),
                texture: None,
            },
        );
    }
    pub fn add_tex_renderable(world: &mut World, id: Entity, mesh: Renderable) {
        world.add_component(id, mesh);
    }
}
