
const VERTEX_SHADER: &str = include_str!("./test_shader.vert");
const FRAGMENT_SHADER: &str  = include_str!("./test_shader.frag");

lazy_static::lazy_static! {
    static ref VERTEX_SPV: Vec<u8> = {
        let mut compiler = shaderc::Compiler::new().unwrap();
        let spv = compiler.compile_into_spirv(VERTEX_SHADER, shaderc::ShaderKind::Vertex, "test_shader.vert", "main", None).unwrap();
        spv.as_binary_u8().into()
    };

    static ref FRAGMENT_SPV: Vec<u8> = {
        let mut compiler = shaderc::Compiler::new().unwrap();
        let spv = compiler.compile_into_spirv(FRAGMENT_SHADER, shaderc::ShaderKind::Fragment, "test_shader.frag", "main", None).unwrap();
        spv.as_binary_u8().into()
    };
}

pub struct Ubershader {
    pub render_pipeline: wgpu::RenderPipeline,
    pub texture_bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
}

impl Ubershader {
    pub fn compile(device: &wgpu::Device, destination_format: wgpu::TextureFormat) -> Self {
        let fs_module = device.create_shader_module(wgpu::util::make_spirv(&FRAGMENT_SPV));
        let vs_module = device.create_shader_module(wgpu::util::make_spirv(&VERTEX_SPV));

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Ubershader texture bind group"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Uint,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        comparison: false,
                    },
                    count: None,
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Ubershader pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout], // fill based on shader
            push_constant_ranges: &[
                wgpu::PushConstantRange {
                    stages: wgpu::ShaderStage::VERTEX,
                    range: 0..(56 * 4),
                },
            ], // fill based on shader
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{
            label: Some("Ubershader pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0f32,
                depth_bias_clamp: 0.0f32,
                clamp_depth: false,
            }),
            color_states: &[
                wgpu::ColorStateDescriptor {
                    format: destination_format,
                    color_blend: wgpu::BlendDescriptor::REPLACE,
                    alpha_blend: wgpu::BlendDescriptor::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }
            ],
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                depth_compare: wgpu::CompareFunction::Greater,
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[
                    super::Vertex::desc(),
                ]
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            label: Some("Ubershader sampler"),
            ..Default::default()
        });

        Ubershader {
            render_pipeline,
            texture_bind_group_layout,
            sampler,
        }
    }

}
