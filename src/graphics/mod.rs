mod resources;
mod deferred_node;
mod octo_node;
mod node_prelude;

use rendy;

#[cfg(test)]
pub mod test_resources;

pub use resources::ResourceManager;

use std::mem::ManuallyDrop;
use {
    rendy::{
        command::{Families},
        factory::{Config, Factory},
        graph::{
            present::PresentNode, render::*, GraphBuilder,
        },
        hal,

        wsi::winit::{EventsLoop, WindowBuilder, Window},
    },
};
use std::sync::Arc;

use octo_runtime::OctoModule;


pub struct Renderer<B: hal::Backend> {
    graph: Option<rendy::graph::Graph<B, Data>>,
}

type Data = dyn crate::scene::traits::Data;

impl<B: hal::Backend> crate::scene::traits::Renderer<Hardware<B>> for Renderer<B> {
    fn create(hardware: &mut Hardware<B>, world: &Data, res: Arc<ResourceManager<B>>) -> Self {
        let graph = fill_render_graph(hardware, world, res);
        Self {
            graph: Some(graph),
        }
    }
    fn run(&mut self, hardware: &mut Hardware<B>, _res: &ResourceManager<B>, world: &Data) {
        match &mut self.graph {
            Some(x) => {
                x.run(&mut hardware.factory, &mut hardware.families, world);
            }
            None => ()
        }
    }

    fn dispose(&mut self, hardware: &mut Hardware<B>, world: &Data) {
        match self.graph.take() {
            Some(x) => {
                x.dispose(&mut hardware.factory, world);
            }
            None => (),
        }
    }
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

impl<B: hal::Backend> crate::scene::traits::Hardware for Hardware<B> {
    type RM = ResourceManager<B>;
    type Renderer = Renderer<B>;
    type Config = i32;

    fn create(_config: &Self::Config) -> Self {
        let conf: rendy::factory::Config = Default::default();
        Self::new(conf)
    }
}

impl<B: hal::Backend> Hardware<B> {
    pub fn new(config: Config) -> Self {
        let (mut factory, families): (Factory<B>, _) = rendy::factory::init(config).unwrap();
        let event_loop = EventsLoop::new();

        let monitor_id = event_loop.get_primary_monitor();

        let window = WindowBuilder::new()
            .with_title("It's Just Game")
            .with_fullscreen(Some(monitor_id))
            .build(&event_loop)
            .unwrap();
        let surface = factory.create_surface(&window);
        let family_id =
            families.as_slice()
                .iter()
                .find(|family| factory.surface_support(family.id(), &surface))
                .map(rendy::command::Family::id).unwrap();

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

pub fn fill_render_graph<'a, B: hal::Backend>(hardware: &mut Hardware<B>, world: &Data, resources: Arc<ResourceManager<B>>) -> rendy::graph::Graph<B, Data> {
    let mut graph_builder = GraphBuilder::<B, Data>::new();

    assert!(hardware.surface.is_some());

    let surface = hardware.surface.take().unwrap();

    let size = hardware.window
        .get_inner_size()
        .unwrap()
        .to_physical(hardware.window.get_hidpi_factor());
    let window_kind = hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1);

    let color = graph_builder.create_image(
        window_kind,
        1,
        hardware.factory.get_surface_format(&surface),
        Some(hal::command::ClearValue::Color([0.0, 1.0, 1.0, 1.0].into())),
    );

    let hdr = graph_builder.create_image(
        hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1),
        1,
        hal::format::Format::Rgba32Sfloat,
        Some(hal::command::ClearValue::Color([0.1, 0.3, 0.4, 1.0].into())),
    );

    let test_color = graph_builder.create_image(
        window_kind,
        1,
        hardware.factory.get_surface_format(&surface),
        Some(hal::command::ClearValue::Color([0.5, 1.0, 1.0, 1.0].into())),
    );

    let depth = graph_builder.create_image(
        window_kind,
        1,
        hal::format::Format::D16Unorm,
        Some(hal::command::ClearValue::DepthStencil(
            hal::command::ClearDepthStencil(1.0, 0),
        )),
    );

    let desc = deferred_node::DeferredNodeDesc { res: resources.clone() };
    let desc2 = octo_node::OctoNodeDesc{res: resources};
    let pass = graph_builder.add_node(
        desc.builder()
            .into_subpass()
            .with_color(hdr)
            .with_depth_stencil(depth)
            .into_pass(),
    );

    let pass2 = graph_builder.add_node(
        desc2.builder()
            .with_image(hdr)
            .into_subpass()
            .with_dependency(pass)
            .with_color(color)
            .into_pass()
    );
    let present_builder = PresentNode::builder(&hardware.factory, surface, color).with_dependency(pass2);

    let frames = present_builder.image_count();

    graph_builder.add_node(present_builder);
    graph_builder
        .with_frames_in_flight(frames)
        .build(&mut hardware.factory, &mut hardware.families, world)
        .unwrap()
}
