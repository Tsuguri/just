mod resources;

use rendy;

#[cfg(test)]
pub mod test_resources;

pub use resources::ResourceManager;

type Backend = rendy::vulkan::Backend;

use failure;
use std::mem::ManuallyDrop;
use crate::scene::traits::ResourceManager as TResourceManager;
use {
    rendy::{
        command::{DrawIndexedCommand, QueueId, RenderPassEncoder, Families},
        factory::{Config, Factory},
        graph::{
            present::PresentNode, render::*, GraphBuilder, GraphContext, NodeBuffer, NodeImage,
        },
        hal::{self, Device as _, PhysicalDevice as _},
        memory::Dynamic,
        mesh::{Mesh, Model, PosColorNorm},
        resource::{Buffer, BufferInfo, DescriptorSet, DescriptorSetLayout, Escape, Handle},
        wsi::winit::{Event, EventsLoop, WindowBuilder, WindowEvent, Window},
        shader::{ShaderKind, SourceLanguage, SourceShaderInfo, SpirvShader},
    },
};
use rendy::shader as rendy_shader;
use std::sync::Arc;
use core::borrow::Borrow;

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("../shader.vert"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/meshes/shader.vert").into(),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("../shader.frag"),
        concat!(env!("CARGO_MANIFEST_DIR"), "/examples/meshes/shader.frag").into(),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}
#[derive(Default)]
struct EmptyNodeDesc {
    res: Arc<ResourceManager>,
}

impl std::fmt::Debug for EmptyNodeDesc {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "EmptyNodeDesc")
    }
}

struct EmptyNode {
    res: Arc<ResourceManager>,
}

impl std::fmt::Debug for EmptyNode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "EmptyNode")
    }
}

impl<B, T> SimpleGraphicsPipelineDesc<B, T> for EmptyNodeDesc
    where
        B: hal::Backend,
        T: ?Sized,
{
    type Pipeline = EmptyNode;

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &T) -> rendy_shader::ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        _factory: &mut Factory<B>,
        _queue: QueueId,
        _aux: &T,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<EmptyNode, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert!(set_layouts.is_empty());

        Ok(EmptyNode {res: self.res})
    }
}


impl<B, T> SimpleGraphicsPipeline<B, T> for EmptyNode
    where
        B: hal::Backend,
        T: ?Sized,
{
    type Desc = EmptyNodeDesc;

    fn draw(&mut self, _layout: &B::PipelineLayout, encoder: RenderPassEncoder<'_, B>, _index: usize, _aux: &T) {
        let p = self.res.get_mesh("heh");
    }

    fn dispose(self, factory: &mut Factory<B>, _aux: &T) {}
}

pub struct Renderer {
    graph: Option<rendy::graph::Graph<Backend, Data>>,
}

type Data = crate::scene::traits::Data;

impl crate::scene::traits::Renderer<Hardware> for Renderer {
    fn create(hardware: &mut Hardware, world: &Data, res: Arc<ResourceManager>) -> Self {
        let graph = fill_render_graph(hardware, world, res);
        Self {
            graph: Some(graph),
        }
    }
    fn run(&mut self, hardware: &mut Hardware, res: &ResourceManager, world: &Data) {
        match &mut self.graph {
            Some(x) => {
                x.run(&mut hardware.factory, &mut hardware.families, world);
            }
            None => ()
        }
    }

    fn dispose(&mut self, hardware: &mut Hardware, world: &Data) {
        match self.graph.take() {
            Some(x) => {
                x.dispose(&mut hardware.factory, world);
            }
            None => (),
        }
    }
}

pub struct Hardware {
    pub window: Window,
    pub event_loop: EventsLoop,
    pub factory: ManuallyDrop<Factory<Backend>>,
    pub families: ManuallyDrop<Families<Backend>>,
    pub surface: Option<rendy::wsi::Surface<Backend>>,
    pub used_family: rendy::command::FamilyId,
}

impl std::ops::Drop for Hardware {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.families);
            ManuallyDrop::drop(&mut self.factory);
        }
    }
}

impl crate::scene::traits::Hardware for Hardware {
    type RM = ResourceManager;
    type Renderer = Renderer;
    type Config = i32;

    fn create(config: &Self::Config) -> Self {
        let conf: rendy::factory::Config = Default::default();
        Self::new(conf)
    }
}

impl Hardware {
    pub fn new(config: Config) -> Self {
        let (mut factory, mut families): (Factory<Backend>, _) = rendy::factory::init(config).unwrap();
        let mut event_loop = EventsLoop::new();

        let monitor_id = event_loop.get_primary_monitor();

        let window = WindowBuilder::new()
            .with_title("Rendy example")
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

pub fn fill_render_graph<'a>(hardware: &mut Hardware, world: &Data, resources: Arc<ResourceManager>) -> rendy::graph::Graph<Backend, Data> {
    let mut graph_builder = GraphBuilder::<Backend, Data>::new();

    assert!(hardware.surface.is_some());
    ;
    let surface = hardware.surface.take().unwrap();

    let size = hardware.window
        .get_inner_size()
        .unwrap()
        .to_physical(hardware.window.get_hidpi_factor());
    let window_kind = hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1);
    let aspect = size.width / size.height;

    let color = graph_builder.create_image(
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
    let desc = EmptyNodeDesc{ res: resources};
    let pass = graph_builder.add_node(
        desc.builder()
            .into_subpass()
            .with_color(color)
            .with_depth_stencil(depth)
            .into_pass(),
    );
    let present_builder = PresentNode::builder(&hardware.factory, surface, color).with_dependency(pass);

    let frames = present_builder.image_count();

    graph_builder.add_node(present_builder);
    graph_builder
        .with_frames_in_flight(frames)
        .build(&mut hardware.factory, &mut hardware.families, world)
        .unwrap()
}
