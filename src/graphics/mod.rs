mod resources;

use rendy;
use super::scene;

#[cfg(test)]
pub mod test_resources;

pub use resources::ResourceManager;

type Backend = rendy::vulkan::Backend;

use failure;
use std::mem::ManuallyDrop;

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
#[derive(Debug, Default)]
struct EmptyNodeDesc {}

#[derive(Debug)]
struct EmptyNode {}

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

        Ok(EmptyNode {})
    }
}


impl<B, T> SimpleGraphicsPipeline<B, T> for EmptyNode
    where
        B: hal::Backend,
        T: ?Sized,
{
    type Desc = EmptyNodeDesc;

    fn draw(&mut self, _layout: &B::PipelineLayout, mut encoder: RenderPassEncoder<'_, B>, _index: usize, _aux: &T) {}

    fn dispose(self, factory: &mut Factory<B>, _aux: &T) {}
}

pub struct Renderer {
    graph: Option<rendy::graph::Graph<Backend, ()>>,
}

impl crate::scene::traits::Renderer<Hardware> for Renderer {
    fn create(hardware: &mut Hardware) -> Self {
        let graph = fill_render_graph(hardware);
        Self {
            graph: Some(graph),
        }
    }
    fn run(&mut self, hardware: &mut Hardware, res: &ResourceManager) {
        match &mut self.graph {
            Some(x) => {
                x.run(&mut hardware.factory, &mut hardware.families, &());
            }
            None => ()
        }
    }

    fn dispose(&mut self, hardware: &mut Hardware) {
        self.graph.take().unwrap().dispose(&mut hardware.factory, &());
    }
}

pub struct Hardware {
    pub window: Window,
    pub event_loop: EventsLoop,
    pub factory: ManuallyDrop<Factory<Backend>>,
    pub families: ManuallyDrop<Families<Backend>>,
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

        Self {
            factory: ManuallyDrop::new(factory),
            families: ManuallyDrop::new(families),
            window,
            event_loop,
        }
    }
}

pub fn fill_render_graph<'a>(hardware: &mut Hardware) -> rendy::graph::Graph<Backend, ()> {
    let mut graph_builder = GraphBuilder::<Backend, ()>::new();
    let surface = hardware.factory.create_surface(&hardware.window);

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
    let pass = graph_builder.add_node(
        EmptyNode::builder()
            .into_subpass()
            .with_color(color)
            .with_depth_stencil(depth)
            .into_pass(),
    );
    let present_builder = PresentNode::builder(&hardware.factory, surface, color).with_dependency(pass);

    let frames = present_builder.image_count();

    graph_builder.add_node(present_builder);
    graph_builder.with_frames_in_flight(frames).build(&mut hardware.factory, &mut hardware.families, &()).unwrap()
}
