use super::node_prelude::*;
use std::collections::HashMap;

use crate::traits::{RenderingData, Value};
use octo_runtime::ValueType;
use std::cell::RefCell;


pub struct PushConstantsBlock {
    buffer: RefCell<Vec<u32>>,
    pub definitions: HashMap<String, (ValueType, usize)>, //type and offset
}

unsafe impl Sync for PushConstantsBlock{}


fn uniform_size(typ: ValueType) -> usize {
    //use ValueType::*;
    match typ {
        ValueType::Float => 16,
        ValueType::Vec2 => 16,
        ValueType::Vec3 => 16,
        ValueType::Vec4 => 16,
        ValueType::Mat3 => 36,
        ValueType::Mat4 => 64,
        ValueType::Int => 16,
        ValueType::Bool => 16,
        _ => panic!(),
    }
}

impl PushConstantsBlock {
    pub fn new(constants: &[(String, ValueType)]) -> PushConstantsBlock {
        let mut offset = 0usize;
        let mut consts = HashMap::new();
        for push_constant in constants {
            consts.insert(push_constant.0.clone(), (push_constant.1, offset));
            offset += uniform_size(push_constant.1)/4;
        }
        let buffer_size = offset;

        let buffer = vec![0;buffer_size];
        PushConstantsBlock{
            buffer: RefCell::new(buffer),
            definitions: consts,
        }
    }

    pub fn clear(&self) {
        self.buffer.borrow_mut().iter_mut().map(|x| *x = 0).count();
    }

    pub fn fill(&self, info_provider: &dyn RenderingData) {
        let mut buff = self.buffer.borrow_mut();
        use rendy::hal::memory;

        for (name, info) in &self.definitions {
            match info_provider.get_rendering_constant(&name) {
                Value::None => {
                    println!("WARNING: There is no data for {} uniform value. Using 0.", name);
                    continue;
                },
                Value::Float(val) => {
                    if info.0 != ValueType::Float {
                        println!("WARNING: Data type mismatch for {}. Engine provided float value, but renderer requested {:?}", name, info.0);
                        continue;
                    }
                    
                    buff[info.1] = memory::cast_slice::<f32, u32>(&[val])[0];
                }
                Value::Vector2(val) => {
                    if info.0 != ValueType::Vec2 {
                        println!("WARNING: Data type mismatch for {}. Engine provided vec2 value, but renderer requested {:?}", name, info.0);
                        continue;
                    }
                    for (offset, value) in memory::cast_slice::<f32, u32>(&val.data).iter().enumerate() {
                        buff[info.1+ offset] = *value;
                    }
                }
                Value::Vector3(val) => {
                    if info.0 != ValueType::Vec3 {
                        println!("WARNING: Data type mismatch for {}. Engine provided vec3 value, but renderer requested {:?}", name, info.0);
                        continue;
                    }
                    for (offset, value) in memory::cast_slice::<f32, u32>(&val.data).iter().enumerate() {
                        buff[info.1+ offset] = *value;
                    }
                }
                Value::Vector4(val) => {
                    if info.0 != ValueType::Vec4 {
                        println!("WARNING: Data type mismatch for {}. Engine provided vec4 value, but renderer requested {:?}", name, info.0);
                        continue;
                    }
                    for (offset, value) in memory::cast_slice::<f32, u32>(&val.data).iter().enumerate() {
                        buff[info.1+ offset] = *value;
                    }
                }
                Value::Matrix3(val) => {
                    if info.0 != ValueType::Mat3 {
                        println!("WARNING: Data type mismatch for {}. Engine provided mat3 value, but renderer requested {:?}", name, info.0);
                        continue;
                    }
                    for (offset, value) in memory::cast_slice::<f32, u32>(&val.data).iter().enumerate() {
                        buff[info.1+ offset] = *value;
                    }
                }
                Value::Matrix4(val) => {
                    if info.0 != ValueType::Mat4 {
                        println!("WARNING: Data type mismatch for {}. Engine provided mat4 value, but renderer requested {:?}", name, info.0);
                        continue;
                    }
                    for (offset, value) in memory::cast_slice::<f32, u32>(&val.data).iter().enumerate() {
                        buff[info.1+ offset] = *value;
                    }
                }
                _ => (),
            }
        }

    }
}

impl Default for PushConstantsBlock {
    fn default() -> PushConstantsBlock {
        PushConstantsBlock {
            buffer: RefCell::new(vec![]),
            definitions: HashMap::new(),
        }
    }
}

#[derive(Default)]
pub struct OctoNodeDesc<B: hal::Backend> {
    pub images: Vec<hal::format::Format>,
    pub res: Arc<ResourceManager<B>>,
    pub vertex_shader: std::cell::RefCell<Vec<u32>>,
    pub fragment_shader: std::cell::RefCell<Vec<u32>>,
    pub stage_name: String,
    pub stage_id: usize,
    pub push_constants_size: usize, 
    pub push_constants_block: Arc<PushConstantsBlock>,
}


pub struct OctoNode<B: hal::Backend> {
    res: Arc<ResourceManager<B>>,
    push_constants_block: Arc<PushConstantsBlock>,
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

impl<B> SimpleGraphicsPipelineDesc<B, dyn RenderingData> for OctoNodeDesc<B>
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

        // fill push constants here

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
            push_constants: vec![
                (rendy::hal::pso::ShaderStageFlags::FRAGMENT, 0..self.push_constants_size as u32),
            ],
        }
    }

    fn load_shader_set(&self, factory: &mut Factory<B>, _aux: &dyn RenderingData) -> ShaderSet<B> {

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
        _data: &dyn RenderingData,
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
            push_constants_block: self.push_constants_block,
            descriptor_set,
            image_sampler,
            image_views,
        })
    }
}

impl<B: hal::Backend> SimpleGraphicsPipeline<B, dyn RenderingData> for OctoNode<B> {
    type Desc = OctoNodeDesc<B>;

    fn prepare(
        &mut self,
        _factory: &Factory<B>,
        _queue: QueueId,
        _set_layouts: &[Handle<DescriptorSetLayout<B>>],
        _index: usize,
        _aux: &dyn RenderingData,
    ) -> PrepareResult {
        PrepareResult::DrawReuse
    }

    fn draw(&mut self, layout: &B::PipelineLayout, mut encoder: RenderPassEncoder<'_, B>, _index: usize, _data: &dyn RenderingData) {
        unsafe {
            let buf = self.push_constants_block.buffer.borrow();

            encoder.push_constants(
                layout,
                hal::pso::ShaderStageFlags::FRAGMENT,
                0,
                &buf,
            );

            encoder.bind_graphics_descriptor_sets(
                layout,
                0,
                std::iter::once(self.descriptor_set.raw()),
                std::iter::empty::<u32>(),
            );
            encoder.draw(0..3, 0..1);
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &dyn RenderingData) {
    }
}
