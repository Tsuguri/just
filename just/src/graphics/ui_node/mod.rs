use super::node_prelude::*;
use crate::ui::*;
use failure;

use just_core::{
    ecs::prelude::*,
    graphics::{
        self,
        hal,
        graph::render::PrepareResult,
    },
    math::*,
};

lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.vert"),
        "ui_node/shader.vert".into(),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.frag"),
        "ui_node/shader.frag".into(),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: graphics::shader::ShaderSetBuilder = graphics::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}

#[derive(Default)]
pub struct UiNodeDesc<B: hal::Backend> {
    pub res: Arc<ResourceManager<B>>,
}

pub struct UiNode<B: hal::Backend> {
    res: Arc<ResourceManager<B>>,
    descriptor_set: Escape<DescriptorSet<B>>,
}

struct UiRenderingData {}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct UiArgs {
    pub(crate) coords: Vec2,
    pub(crate) dimensions: Vec2,
    pub(crate) tex_coord_bounds: Vec4,
    pub(crate) color: Vec4,
    pub(crate) color_bias: Vec4,
}
impl<B: hal::Backend> std::fmt::Debug for UiNodeDesc<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "UiNodeDesc")
    }
}

impl<B: hal::Backend> std::fmt::Debug for UiNode<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "UiNode")
    }
}

impl<B> SimpleGraphicsPipelineDesc<B, World> for UiNodeDesc<B>
where
    B: hal::Backend,
{
    type Pipeline = UiNode<B>;

    fn colors(&self) -> Vec<hal::pso::ColorBlendDesc> {
        vec![hal::pso::ColorBlendDesc {
            mask: hal::pso::ColorMask::ALL,
            blend: None,
        }]
    }

    fn input_assembler(&self) -> hal::pso::InputAssemblerDesc {
        hal::pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip)
    }

    fn layout(&self) -> Layout {
        //vec2 inverse_window_size;
        //vec2 coords;
        //vec2 dimensions;
        //vec4 tex_coord_bounds;
        //vec4 color;
        //vec4 color_bias;
        let push_constants = vec![
            // vec2, 4 bytes each component
            (
                graphics::hal::pso::ShaderStageFlags::VERTEX,
                0..((2 + 2 + 2 + 4 + 4 + 4) * 4),
            ),
        ];
        let sets = vec![SetLayout {
            bindings: vec![
                hal::pso::DescriptorSetLayoutBinding {
                    binding: 0,
                    ty: hal::pso::DescriptorType::SampledImage,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
                hal::pso::DescriptorSetLayoutBinding {
                    binding: 1,
                    ty: hal::pso::DescriptorType::Sampler,
                    count: 1,
                    stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                    immutable_samplers: false,
                },
            ],
        }];
        Layout {
            sets,
            push_constants,
        }
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &World) -> ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _data: &World,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<UiNode<B>, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let texture_id = self.res.get_texture("tex1").unwrap();
        let texture = self.res.get_real_texture(texture_id);

        let descriptor_set = factory
            .create_descriptor_set(set_layouts[0].clone())
            .unwrap();
        unsafe {
            factory.device().write_descriptor_sets(vec![
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Image(
                        texture.texture.view().raw(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )],
                },
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 1,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Sampler(
                        texture.texture.sampler().raw(),
                    )],
                },
            ]);
        }

        Ok(UiNode {
            res: self.res,
            descriptor_set,
        })
    }
}

unsafe fn push_ui_consts<B: hal::Backend>(
    encoder: &mut RenderPassEncoder<'_, B>,
    consts: &UiArgs,
    layout: &B::PipelineLayout,
) {
    //vec2 inverse_window_size;
    //vec2 coords;
    //vec2 dimensions;
    //vec4 tex_coord_bounds;
    //vec4 color;
    //vec4 color_bias;
    let tc_bounds_offset: u32 = 0 * 4;
    let color_offset: u32 = 4 * 4;
    let bias_offset: u32 = 8 * 4;

    let inverse_window_size_offset: u32 = 12 * 4;
    let coords_offset: u32 = 14 * 4;
    let dimensions_offset: u32 = 16 * 4;

    let inverse_window_size = Vec2::new(1.0f32 / 2560.0f32, 1.0f32 / 1080.0f32);

    encoder.push_constants(
        layout,
        hal::pso::ShaderStageFlags::VERTEX,
        inverse_window_size_offset,
        hal::memory::cast_slice::<f32, u32>(&inverse_window_size.data),
    );
    encoder.push_constants(
        layout,
        hal::pso::ShaderStageFlags::VERTEX,
        coords_offset,
        hal::memory::cast_slice::<f32, u32>(&consts.coords.data),
    );
    encoder.push_constants(
        layout,
        hal::pso::ShaderStageFlags::VERTEX,
        dimensions_offset,
        hal::memory::cast_slice::<f32, u32>(&consts.dimensions.data),
    );
    encoder.push_constants(
        layout,
        hal::pso::ShaderStageFlags::VERTEX,
        tc_bounds_offset,
        hal::memory::cast_slice::<f32, u32>(&consts.tex_coord_bounds.data),
    );
    encoder.push_constants(
        layout,
        hal::pso::ShaderStageFlags::VERTEX,
        color_offset,
        hal::memory::cast_slice::<f32, u32>(&consts.color.data),
    );
    encoder.push_constants(
        layout,
        hal::pso::ShaderStageFlags::VERTEX,
        bias_offset,
        hal::memory::cast_slice::<f32, u32>(&consts.color_bias.data),
    );
}

impl<B> SimpleGraphicsPipeline<B, World> for UiNode<B>
where
    B: hal::Backend,
{
    type Desc = UiNodeDesc<B>;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &World,
    ) -> PrepareResult {
        PrepareResult::DrawRecord
    }

    fn draw(
        &mut self,
        layout: &B::PipelineLayout,
        mut encoder: RenderPassEncoder<'_, B>,
        _index: usize,
        data: &World,
    ) {
        unsafe {
            let ui_system = data.resources.get::<UiSystem>();
            let ui_system = match ui_system {
                None => return,
                Some(x) => x,
            };

            let q = <(Read<UiTransform>, Read<UiRenderable>)>::query();

            for (index, (id, (transform, renderable))) in
                q.iter_entities_immutable(&data).enumerate()
            {
                let lt = ui_system.layout.layout(transform.node).unwrap();
                let pos = Vec2::new(lt.location.x, lt.location.y);
                let size = Vec2::new(lt.size.width, lt.size.height);

                match *renderable {
                    UiRenderable::Rect(tex_id) => {
                        let args = UiArgs {
                            // shader wants center of element
                            coords: pos + size * 0.5f32,
                            dimensions: size,
                            tex_coord_bounds: Vec4::new(0.0f32, 0.0f32, 1.0f32, 1.0f32),
                            color: Vec4::new(1.0f32, 1.0f32, 1.0f32, 1.0f32),
                            color_bias: Vec4::new(1.0f32, 1.0f32, 1.0f32, 1.0f32),
                        };
                        push_ui_consts(&mut encoder, &args, layout);

                        let tex = self.res.get_real_texture(tex_id);
                        encoder.bind_graphics_descriptor_sets(
                            layout,
                            0,
                            std::iter::once(tex.desc.raw()),
                            std::iter::empty::<u32>(),
                        );
                        encoder.draw(0..4, 0..2);
                    }
                }
            }
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &World) {}
}

/*
impl<B> RenderGroupDesc<B, World> for UiNodeDesc<B>    where B: hal::Backend {
    fn build(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: Subpass<B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>
    ) -> Result<Box<dyn RenderGroup<B, World> + 'static>, failure::Error> {

        let layout: Vec<&B::DescriptorSetLayout> = vec![];

        let (pipeline, pipeline_layout) = build_ui_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            layout,
        )?;

        Ok(Box::new(UiNode::<B> {
            pipeline,
            pipeline_layout,
        }))
    }
}
use crate::math::*;
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C, align(4))]
pub(crate) struct UiArgs {
    pub(crate) coords: Vec2,
    pub(crate) dimensions: Vec2,
    pub(crate) tex_coord_bounds: Vec4,
    pub(crate) color: Vec4,
    pub(crate) color_bias: Vec4,
}

impl AsVertex for UiArgs {
    fn vertex() -> rendy::mesh::VertexFormat {
        rendy::mesh::VertexFormat::new((
            (hal::format::Format::Rg32Sfloat, "coords"),
            (hal::format::Format::Rg32Sfloat, "dimensions"),
            (hal::format::Format::Rgba32Sfloat, "tex_coord_bounds"),
            (hal::format::Format::Rgba32Sfloat, "color"),
            (hal::format::Format::Rgba32Sfloat, "color_bias"),
        ))
    }
}

fn push_vertex_desc(
    elements: &[hal::pso::Element<hal::format::Format>],
    stride: hal::pso::ElemStride,
    rate: hal::pso::VertexInputRate,
    vertex_buffers: &mut Vec<hal::pso::VertexBufferDesc>,
    attributes: &mut Vec<hal::pso::AttributeDesc>,
) {
    let index = vertex_buffers.len() as hal::pso::BufferIndex;

    vertex_buffers.push(hal::pso::VertexBufferDesc {
        binding: index,
        stride,
        rate,
    });

    let mut location = attributes.last().map_or(0, |a| a.location + 1);
    for &element in elements {
        attributes.push(hal::pso::AttributeDesc {
            location,
            binding: index,
            element,
        });
        location += 1;
    }
}

fn build_ui_pipeline<B: hal::Backend>(factory: &mut Factory<B>, subpass: Subpass<B>, framebuffer_width: u32, framebuffer_height: u32,layouts: Vec<&B::DescriptorSetLayout>) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;


    //let vertex_desc = [(UiArgs::vertex(), hal::pso::VertexInputRate::Instance(1))];
    let mut vertex_buffers = vec![];
    let mut attributes = vec![];

    let desc = UiArgs::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Instance(1));

    push_vertex_desc(&desc.0, desc.1, desc.2, &mut vertex_buffers, &mut attributes);


    let rekt = hal::pso::Rect {
        x: 0, y: 0,
        w: framebuffer_width as i16,
        h: framebuffer_height as i16
    };

    let pipeline_desc = GraphicsPipelineDesc {
            vertex_buffers,
            attributes,
            input_assembler: hal::pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip),
            shaders: SHADERS.build(factory, Default::default()).unwrap().raw().unwrap(),
            layout: &pipeline_layout,
            subpass: subpass,
            baked_states: hal::pso::BakedStates{
                viewport: Some(hal::pso::Viewport{
                    rect: rekt,
                    depth: 0.0..1.0,
                }),
                scissor: Some(rekt),
                ..Default::default()
            },
            blender: hal::pso::BlendDesc {
                targets: vec![hal::pso::ColorBlendDesc{
                             mask: hal::pso::ColorMask::ALL,
                             blend: Some(hal::pso::BlendState::ALPHA),
                }],
                ..Default::default()
            },
            rasterizer: hal::pso::Rasterizer::FILL,
            flags: hal::pso::PipelineCreationFlags::empty(),
            parent: hal::pso::BasePipeline::None,
            depth_stencil: hal::pso::DepthStencilDesc::default(),
            multisampling: None,
    };
    let mut pipelines = unsafe {
            factory
                .device()
                .create_graphics_pipelines(&[pipeline_desc], None)
        };

        if let Some(err) = pipelines.iter().find_map(|p| p.as_ref().err().cloned()) {
            for p in pipelines.drain(..).filter_map(Result::ok) {
                unsafe {
                    factory.destroy_graphics_pipeline(p);
                }
            }
            failure::bail!(err);
        }

    let mut pipes: Vec<_> = pipelines.into_iter().map(|p| p.unwrap()).collect();

    Ok((pipes.remove(0), pipeline_layout))

}

impl<B> RenderGroup<B, World> for UiNode<B>
    where B: hal::Backend {
    fn prepare(
            &mut self,
            factory: &Factory<B>,
            _queue: QueueId,
            index: usize,
            _subpass: hal::pass::Subpass<'_, B>,
            resources: &World,
        ) -> PrepareResult {
        PrepareResult::DrawReuse
    }
     fn draw_inline(
            &mut self,
            mut encoder: RenderPassEncoder<'_, B>,
            index: usize,
            _subpass: hal::pass::Subpass<'_, B>,
            _resources: &World,
        ) {
     }
    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        /*unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }*/
    }
}
*/
