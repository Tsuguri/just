use super::node_prelude::*;

use crate::traits::{RenderingData};


#[derive(Default)]
pub struct OctoNodeDesc<B: hal::Backend> {
    pub images: Vec<hal::format::Format>,
    pub res: Arc<ResourceManager<B>>,
    pub vertex_shader: std::cell::RefCell<Vec<u32>>,
    pub fragment_shader: std::cell::RefCell<Vec<u32>>,
    pub stage_name: String,
    pub stage_id: usize,
}


pub struct OctoNode<B: hal::Backend> {
    res: Arc<ResourceManager<B>>,
    descriptor_set: Escape<DescriptorSet<B>>,
    image_sampler: Escape<Sampler<B>>,
    image_views: Vec<Escape<ImageView<B>>>,
}

impl<B: hal::Backend> std::fmt::Debug for OctoNodeDesc<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "OctoNodeDesc")
    }
}

impl<B: hal::Backend> std::fmt::Debug for OctoNode<B> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(formatter, "OctoNode")
    }
}

impl<B> SimpleGraphicsPipelineDesc<B, RenderingData> for OctoNodeDesc<B>
    where B: hal::Backend {
    type Pipeline = OctoNode<B>;

    fn colors(&self) -> Vec<hal::pso::ColorBlendDesc> {
        vec![
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: None,
            },
        ]
    }
    fn images(&self) -> Vec<ImageAccess> {
        std::iter::repeat(ImageAccess {
            access: hal::image::Access::SHADER_READ,
            usage: hal::image::Usage::SAMPLED,
            layout: hal::image::Layout::ShaderReadOnlyOptimal,
            stages: hal::pso::PipelineStage::FRAGMENT_SHADER,
        }).take(self.images.len()).collect()
    }

    fn depth_stencil(&self) -> Option<hal::pso::DepthStencilDesc> {
        None
    }

    fn layout(&self) -> Layout {
        Layout {
            sets: vec![SetLayout {
                bindings: vec![
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 0,
                        ty: hal::pso::DescriptorType::Sampler,
                        count: 1,
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                    hal::pso::DescriptorSetLayoutBinding {
                        binding: 1,
                        ty: hal::pso::DescriptorType::SampledImage,
                        count: self.images.len(),
                        stage_flags: hal::pso::ShaderStageFlags::FRAGMENT,
                        immutable_samplers: false,
                    },
                ],
            }],
            push_constants: Vec::new(),
        }
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &RenderingData) -> ShaderSet<B> {

        let fragment_spirv= self.fragment_shader.replace(vec![]);
        let vertex_spirv= self.vertex_shader.replace(vec![]);

       let fragment = SpirvShader::new(fragment_spirv, hal::pso::ShaderStageFlags::FRAGMENT, "main");
        let vertex = SpirvShader::new(vertex_spirv, hal::pso::ShaderStageFlags::VERTEX, "main");

        let shaders: rendy::shader::ShaderSetBuilder = rendy::shader::ShaderSetBuilder::default()
            .with_vertex(&vertex).unwrap()
            .with_fragment(&fragment).unwrap();
        shaders.build(factory, Default::default()).unwrap()
    }

    fn build<'a>(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        _data: &RenderingData,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<OctoNode<B>, failure::Error> {
        use std::iter::Iterator as _;
        assert!(buffers.is_empty());
        assert_eq!(images.len(), self.images.len());
        assert_eq!(set_layouts.len(), 1);

        let image_sampler =
            factory.create_sampler(SamplerInfo::new(Filter::Nearest, WrapMode::Clamp))?;

        let mut image_views = Vec::with_capacity(self.images.len());

        for (id, image_format) in self.images.iter().enumerate() {
            let image_handle = ctx
                .get_image(images[id].id)
                .ok_or(failure::format_err!("Tonemapper HDR image missing"))?;

            let image_view = factory
                .create_image_view(
                    image_handle.clone(),
                    ImageViewInfo {
                        view_kind: ViewKind::D2,
                        format: *image_format,
                        swizzle: hal::format::Swizzle::NO,
                        range: images[0].range.clone(),
                    },
                )
                .map_err(|_err| failure::format_err!("Could not create tonemapper input image view"))?;
            image_views.push(image_view);
        }


        let descriptor_set = factory
            .create_descriptor_set(set_layouts[0].clone())?;
        unsafe {
            let mut descriptor_set_operations =
            vec![
                hal::pso::DescriptorSetWrite {
                    set: descriptor_set.raw(),
                    binding: 0,
                    array_offset: 0,
                    descriptors: vec![hal::pso::Descriptor::Sampler(image_sampler.raw())],
                },
            ];

            for (id, image) in image_views.iter().enumerate() {

                descriptor_set_operations.push(
                    hal::pso::DescriptorSetWrite {
                        set: descriptor_set.raw(),
                        binding: 1,
                        array_offset: id,
                        descriptors: vec![hal::pso::Descriptor::Image(
                            image.raw(),
                            hal::image::Layout::ShaderReadOnlyOptimal,
                        )],
                    }
                );
            }
            factory.device().write_descriptor_sets(descriptor_set_operations);
        }

        Result::Ok(OctoNode {
            res: self.res,
            descriptor_set,
            image_sampler,
            image_views,
        })
    }
}

impl<B: hal::Backend> SimpleGraphicsPipeline<B, RenderingData> for OctoNode<B> {
    type Desc = OctoNodeDesc<B>;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &RenderingData,
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw(&mut self, layout: &B::PipelineLayout, mut encoder: RenderPassEncoder<'_, B>, _index: usize, _data: &RenderingData) {
        unsafe {
            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.descriptor_set.raw()),
                std::iter::empty::<u32>(),
            );
            encoder.draw(0..3, 0..1);
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &RenderingData) {
    }
}
