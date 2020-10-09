use super::node_prelude::*;

use super::octo_node::{RenderingConstants, Value};
use crate::TRenderable as Renderable;
use just_core::hierarchy::TransformHierarchy;
use just_core::ecs::prelude::*;
use just_core::math;
use just_assets::{Handle as AssetHandle, AssetStorage};
use super::Mesh;
use super::RTexture;

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
    _phantom: std::marker::PhantomData<B>,
}

impl<B: hal::Backend> DeferredNodeDesc<B> {
    pub fn new() -> Self {
        Self{
            _phantom: Default::default(),
        }
    }
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


pub struct DeferredNode<B: hal::Backend> {
    descriptor_set: Escape<DescriptorSet<B>>,
    renderables_buffer: Option<Vec<(AssetHandle<rendy::mesh::Mesh<B>>, Option<AssetHandle<RTexture<B>>>, math::Matrix)>>,
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

impl<B> SimpleGraphicsPipelineDesc<B, World> for DeferredNodeDesc<B>
where
    B: hal::Backend,
{
    type Pipeline = DeferredNode<B>;

    fn colors(&self) -> Vec<hal::pso::ColorBlendDesc> {
        vec![
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: Some(hal::pso::BlendState::ALPHA),
            },
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: Some(hal::pso::BlendState::ALPHA),
            },
            hal::pso::ColorBlendDesc {
                mask: hal::pso::ColorMask::ALL,
                blend: Some(hal::pso::BlendState::ALPHA),
            },
        ]
    }
    fn vertices(
        &self,
    ) -> Vec<(
        Vec<hal::pso::Element<hal::format::Format>>,
        hal::pso::ElemStride,
        hal::pso::VertexInputRate,
    )> {
        vec![PosNormTex::vertex().gfx_vertex_input_desc(hal::pso::VertexInputRate::Vertex)]
    }
    fn layout(&self) -> Layout {
        let push_constants = vec![(rendy::hal::pso::ShaderStageFlags::VERTEX, 0..(56 * 4))];
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
        data: &World,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>,
        set_layouts: &[Handle<DescriptorSetLayout<B>>],
    ) -> Result<DeferredNode<B>, failure::Error> {
        assert!(buffers.is_empty());
        assert!(images.is_empty());
        assert_eq!(set_layouts.len(), 1);

        let texture_storage = data.resources.get::<AssetStorage<RTexture<B>>>().unwrap();
        let texture_handle = texture_storage.get_handle("tex1").unwrap();
        let texture = texture_storage.get_value(&texture_handle).unwrap();


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

        Ok(DeferredNode {
            descriptor_set,
            renderables_buffer: None,
        })
    }
}

impl<B> SimpleGraphicsPipeline<B, World> for DeferredNode<B>
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
            //println!("deferred rendering");
            let vertex = [PosNormTex::vertex()];

            // offset of model matrix in push constants
            // 16 fields, 4 bytes each, 2 matrices before.
            let model_offset: u32 = 16 * 4 * 2;

            {
                let view_offset: u32 = 0;
                let projection_offset: u32 = 16 * 4;

                let view = match RenderingConstants::get_rendering_constant(data, "view_mat") {
                    Value::Matrix4(mat) => mat,
                    _ => panic!("Internal renderer error E02"),
                };

                encoder.push_constants(
                    layout,
                    hal::pso::ShaderStageFlags::VERTEX,
                    view_offset,
                    hal::memory::cast_slice::<f32, u32>(&view.data),
                );

                let projection =
                    match RenderingConstants::get_rendering_constant(data, "projection_mat") {
                        Value::Matrix4(mat) => mat,
                        _ => panic!("internal renderer error E01"),
                    };
                encoder.push_constants(
                    layout,
                    hal::pso::ShaderStageFlags::VERTEX,
                    projection_offset,
                    hal::memory::cast_slice::<f32, u32>(&projection.data),
                );
            }

            let buf = self.renderables_buffer.take();

            let buf = {
                let query = <Read<Renderable<B>>>::query();

                let mut buf = match buf {
                    Some(mut vec) => {
                        vec.clear();
                        vec
                    }
                    None => Vec::new(),
                };

                for (entity_id, mesh) in query.iter_entities_immutable(data) {
                    let mat = TransformHierarchy::get_global_matrix(data, entity_id);
                    match mesh.mesh_handle {
                        None =>{
                            println!("\trenderable with empty mesh");

                        },
                        Some(x) => {
                            buf.push((x, mesh.texture_handle, mat));
                        }
                    }
                }
                println!("done with renderables");
                buf
            };

            let mesh_storage = data.resources.get::<AssetStorage<rendy::mesh::Mesh<B>>>().unwrap();
            let texture_storage = data.resources.get::<AssetStorage<RTexture<B>>>().unwrap();

            for renderable in &buf {
                let model = renderable.2;

                encoder.push_constants(
                    layout,
                    hal::pso::ShaderStageFlags::VERTEX,
                    model_offset,
                    hal::memory::cast_slice::<f32, u32>(&model.data),
                );
                let mesh = mesh_storage.get_value(&renderable.0);
                
                match mesh {
                    None => (),
                    Some(x) => {
                        match renderable.1 {
                            None => {
                                encoder.bind_graphics_descriptor_sets(
                                    layout,
                                    0,
                                    std::iter::once(self.descriptor_set.raw()),
                                    std::iter::empty::<u32>(),
                                );
                            }
                            Some(x) => {
                                let tex = texture_storage.get_value(&x);
                                match tex {
                                    None => (),
                                    Some(tex) => {
                                        encoder.bind_graphics_descriptor_sets(
                                            layout,
                                            0,
                                            std::iter::once(tex.desc.raw()),
                                            std::iter::empty::<u32>(),
                                        );
                                    }
                                }
                            }
                        };
                        x.bind_and_draw(0, &vertex, 0..1, &mut encoder).unwrap();
                    }
                }
            }
            self.renderables_buffer = Some(buf);
        }
    }

    fn dispose(self, _factory: &mut Factory<B>, _aux: &World) {}
}
