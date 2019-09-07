use super::node_prelude::*;



lazy_static::lazy_static! {
    static ref VERTEX: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.vert"),
        "deferred_node/shader.vert".into(),
        ShaderKind::Vertex,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref FRAGMENT: SpirvShader = SourceShaderInfo::new(
        include_str!("shader.frag"),
        "deferred_node/shader.frag".into(),
        ShaderKind::Fragment,
        SourceLanguage::GLSL,
        "main",
    ).precompile().unwrap();

    static ref SHADERS: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
        .with_vertex(&*VERTEX).unwrap()
        .with_fragment(&*FRAGMENT).unwrap();
}

#[derive(Default)]
pub struct DeferredNodeDesc<B: hal::Backend> {
    pub res: Arc<ResourceManager<B>>,
}

impl<B: hal::Backend> DeferredNodeDesc<B> {
    pub fn position_format(&self) -> hal::format::Format {
        hal::format::Format::Rgba32Sfloat
    }

    pub fn normal_format(&self) -> hal::format::Format {
        hal::format::Format::Rgba32Sfloat
    }

    pub fn albedo_format(&self) -> hal::format::Format {
        hal::format::Format::Rgba32Sfloat
    }
}

use super::Hardware;
use crate::scene::traits::{MeshId, TextureId};

pub struct DeferredNode<B: hal::Backend> {
    res: Arc<ResourceManager<B>>,
    descriptor_set: Escape<DescriptorSet<B>>,
    renderables_buffer: Option<Vec<(MeshId, Option<TextureId>, crate::scene::math::Matrix)>>,
}

impl<B: hal::Backend> std::fmt::Debug for DeferredNodeDesc<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "DeferredNodeDesc")
    }
}

impl<B: hal::Backend> std::fmt::Debug for DeferredNode<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "DeferredNode")
    }
}

impl<B> SimpleGraphicsPipelineDesc<B, Data> for DeferredNodeDesc<B>
    where
        B: hal::Backend,
{
    type Pipeline = DeferredNode<B>;

    fn colors(&self) -> Vec<hal::pso::ColorBlendDesc> {
        vec![
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: None,
            },
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: None,
            },
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: None,
            },
        ]
    }
    fn vertices(&self) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        vec![PosNormTex::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
    }
    fn layout(&self) -> Layout {
        let push_constants = vec![(rendy::hal::pso::ShaderStageFlags::VERTEX, 0..(56 * 4))];
        let sets = vec![
            SetLayout {
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
            }
        ];
        Layout {
            sets,
            push_constants,
        }
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &Data) -> ShaderSet<B> {
        SHADERS.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _data: &Data,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<DeferredNode<B>, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let texture_id = self.res.get_texture("creature").unwrap();
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
                        texture.view().raw(),
                        hal::image::Layout::ShaderReadOnlyOptimal,
                    )],
                },
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 1,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Sampler(texture.sampler().raw())],
                },
            ]);
        }


        Ok(DeferredNode { res: self.res, descriptor_set, renderables_buffer: None })
    }
}


impl<B> SimpleGraphicsPipeline<B, Data> for DeferredNode<B>
    where
        B: hal::Backend,
{
    type Desc = DeferredNodeDesc<B>;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &Data,
    ) -> PrepareResult {
        PrepareResult::DrawRecord
    }

    fn draw(&mut self, layout: &B::PipelineLayout, mut encoder: RenderPassEncoder<'_, B>, _index: usize, data: &Data) {
        unsafe {
            //println!("deferred rendering");
            let vertex = [PosNormTex::vertex()];

            // offset of model matrix in push constants
            // 16 fields, 4 bytes each, 2 matrices before.
            let model_offset: u32 = 16 * 4 * 2;

            {
                encoder.bind_graphics_descriptor_sets(
                    layout,
                    0,
                    std::iter::once(self.descriptor_set.raw()),
                    std::iter::empty::<u32>(),
                );
            }

            {
                let view_offset: u32 = 0;
                let projection_offset: u32 = 16 * 4;

                let view = data.get_view_matrix();

                encoder.push_constants(
                    layout,
                    hal::pso::ShaderStageFlags::VERTEX,
                    view_offset,
                    hal::memory::cast_slice::<f32, u32>(&view.data),
                );

                let projection = data.get_projection_matrix();
                encoder.push_constants(
                    layout,
                    hal::pso::ShaderStageFlags::VERTEX,
                    projection_offset,
                    hal::memory::cast_slice::<f32, u32>(&projection.data),
                );
            }

            let buf = self.renderables_buffer.take();

            let buf = data.get_renderables(buf);

            for renderable in &buf {
                let model = renderable.2;

                encoder.push_constants(
                    layout,
                    hal::pso::ShaderStageFlags::VERTEX,
                    model_offset,
                    hal::memory::cast_slice::<f32, u32>(&model.data),
                );
                let mesh = self.res.get_real_mesh(renderable.0);
                mesh.bind_and_draw(0, &vertex, 0..1, &mut encoder).unwrap();

            }
            self.renderables_buffer = Some(buf);

        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &Data) {}
}
