mod resources;

use rendy;

#[cfg(test)]
pub mod test_resources;

pub use resources::ResourceManager;

//type Backend = rendy::vulkan::Backend;

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
        mesh::{Mesh, Model, PosNormTex, AsVertex},
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
struct EmptyNodeDesc<B: hal::Backend> {
    res: Arc<ResourceManager<B>>,
}

impl<B: hal::Backend> std::fmt::Debug for EmptyNodeDesc<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "EmptyNodeDesc")
    }
}

struct EmptyNode<B: hal::Backend> {
    vertex: Option<Escape<Buffer<B>>>,
    res: Arc<ResourceManager<B>>,
}

impl<B: hal::Backend> std::fmt::Debug for EmptyNode<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "EmptyNode")
    }
}

impl<B, T> SimpleGraphicsPipelineDesc<B, T> for EmptyNodeDesc<B>
    where
        B: hal::Backend,
        T: ?Sized,
{
    type Pipeline = EmptyNode<B>;

    fn vertices(&self) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        return vec![PosNormTex::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)];
    }

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
    ) -> Result<EmptyNode<B>, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert!(set_layouts.is_empty());

        Ok(EmptyNode { res: self.res, vertex: None })
    }
}


impl<B, T> SimpleGraphicsPipeline<B, T> for EmptyNode<B>
    where
        B: hal::Backend,
        T: ?Sized,
{
    type Desc = EmptyNodeDesc<B>;

    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &T,
    ) -> PrepareResult {
        if self.vertex.is_none() {

                let vbuf_size = PosNormTex::vertex().stride as u64 * 6;

            let mut vbuf = factory
                .create_buffer(
                    BufferInfo {
                        size: vbuf_size,
                        usage: hal::buffer::Usage::VERTEX,
                    },
                    Dynamic,
                )
                .unwrap();

            unsafe {
                // Fresh buffer.
                factory
                    .upload_visible_buffer(
                        &mut vbuf,
                        0,
                        &[
                            PosNormTex {
                                position: [0.0, -0.5, 0.0].into(),
                                tex_coord: [1.0, 0.0].into(),
                                normal: [1.0, 1.0, 1.0].into(),
                            },
                            PosNormTex {
                                position: [0.5, 0.5, 0.0].into(),
                                tex_coord: [1.0, 0.0].into(),
                                normal: [1.0, 0.0, 1.0].into(),
                            },
                            PosNormTex {
                                position: [0.0, 0.5, 0.0].into(),
                                tex_coord: [1.0, 0.0].into(),
                                normal: [0.0, 1.0, 1.0].into(),
                            },



                            PosNormTex {
                                position: [0.0, -0.5, 0.0].into(),
                                normal: [1.0, 1.0, 1.0].into(),
                                tex_coord: [1.0, 0.0].into(),
                            },
                            PosNormTex {
                                position: [0.5, 0.9, 0.0].into(),
                                normal: [1.0, 0.0, 1.0].into(),
                                tex_coord: [1.0, 0.0].into(),
                            },
                            PosNormTex {
                                position: [0.0, 0.5, 0.0].into(),
                                normal: [0.0, 1.0, 1.0].into(),
                                tex_coord: [1.0, 0.0].into(),
                            },


                        ],
                    )
                    .unwrap();
            }

            self.vertex = Some(vbuf);
        }

        PrepareResult::DrawReuse
    }

    fn draw(&mut self, _layout: &B::PipelineLayout, mut encoder: RenderPassEncoder<'_, B>, _index: usize, _aux: &T) {
        //let p = self.res.get_mesh("plane").unwrap();

        //let monkey_mesh = self.res.get_real_mesh(p);

        //let vbuf = self.vertex.as_ref().unwrap();
        unsafe {
//            let vertex = [PosNormTex::vertex()];
            //encoder.bind_vertex_buffers(0, Some((vbuf.raw(), 0)));
            //encoder.draw(0..3, 0..1);

            //monkey_mesh.bind_and_draw(0, &vertex, 0..1, &mut encoder);
//            monkey_mesh.bind(0, &vertex, &mut encoder);
//            encoder.draw(0..3, 0..1);



//            let vbuf = self.vertex.as_ref().unwrap();
//            encoder.bind_vertex_buffers(0, Some((vbuf.raw(), 0)));
//            encoder.draw(0..6, 0..1);


            let p = self.res.get_mesh("plane").unwrap();

            let monkey_mesh = self.res.get_real_mesh(p);
            let vertex = [PosNormTex::vertex()];
            monkey_mesh.bind_and_draw(0, &vertex, 0..1, &mut encoder).unwrap();
            println!("rendering {} indices", monkey_mesh.len());
        }
    }

    fn dispose(self, factory: &mut Factory<B>, _aux: &T) {}
}

pub struct Renderer<B: hal::Backend> {
    graph: Option<rendy::graph::Graph<B, Data>>,
}

type Data = crate::scene::traits::Data;

impl<B: hal::Backend> crate::scene::traits::Renderer<Hardware<B>> for Renderer<B> {
    fn create(hardware: &mut Hardware<B>, world: &Data, res: Arc<ResourceManager<B>>) -> Self {
        let graph = fill_render_graph(hardware, world, res);
        Self {
            graph: Some(graph),
        }
    }
    fn run(&mut self, hardware: &mut Hardware<B>, res: &ResourceManager<B>, world: &Data) {
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

    fn create(config: &Self::Config) -> Self {
        let conf: rendy::factory::Config = Default::default();
        Self::new(conf)
    }
}

impl<B: hal::Backend> Hardware<B> {
    pub fn new(config: Config) -> Self {
        let (mut factory, mut families): (Factory<B>, _) = rendy::factory::init(config).unwrap();
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

pub fn fill_render_graph<'a, B: hal::Backend>(hardware: &mut Hardware<B>, world: &Data, resources: Arc<ResourceManager<B>>) -> rendy::graph::Graph<B, Data> {
    let mut graph_builder = GraphBuilder::<B, Data>::new();

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
    let desc = EmptyNodeDesc { res: resources };
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
