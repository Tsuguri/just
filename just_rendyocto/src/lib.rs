pub use rendy;
pub use octo_runtime;
use rendy::{
    command::Families,
    factory::{Config, Factory},
    graph::{present::PresentNode, render::*, GraphBuilder},
    hal,
    hal::device::Device,
    wsi::winit::{EventsLoop, Window, WindowBuilder},
    mesh::{Mesh as RMesh, MeshBuilder, Normal, PosNormTex, Position, TexCoord},
    resource::Escape,
    texture::image,
};
use std::mem::ManuallyDrop;
use just_core::ecs::prelude::*;
use just_core::math::{Vec2, Vec3, Quat};
use std::sync::Arc;
use octo_runtime::OctoModule;
use just_assets::*;
use wavefront_obj::obj;

pub type Mesh = RMesh<rendy::vulkan::Backend>;
pub type Texture = RTexture<rendy::vulkan::Backend>;

pub struct RTexture<B: hal::Backend> {
    pub texture: rendy::texture::Texture<B>,
    pub desc: rendy::resource::DescriptorSet<B>,
}

#[derive(Clone)]
pub struct CameraData {
    pub position: Vec3,
    pub rotation: Quat,
}

#[derive(Clone)]
pub struct ViewportData {
    pub camera_lens_height: f32,
    pub height: f32,
    pub width: f32,
    pub ratio: f32,
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

impl<B: hal::Backend> Hardware<B> {
    pub fn create() -> Self {
        let conf: rendy::factory::Config = Default::default();
        Self::new(conf)
    }
    pub fn new(config: Config) -> Self {
        let (mut factory, families): (Factory<B>, _) = rendy::factory::init(config).unwrap();
        let mut event_loop = EventsLoop::new();
        event_loop.poll_events(|_| ());

        let monitor_id = event_loop.get_primary_monitor();

        let window = WindowBuilder::new()
            .with_title("It's Just Game")
            .with_fullscreen(Some(monitor_id))
            .build(&event_loop)
            .unwrap();
        let surface = factory.create_surface(&window);
        let family_id = families
            .as_slice()
            .iter()
            .find(|family| factory.surface_support(family.id(), &surface))
            .map(rendy::command::Family::id)
            .unwrap();

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

pub mod deferred_node;
pub mod octo_node;
pub mod node_prelude;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TRenderable<B: hal::Backend> {
    pub mesh_handle: Option<Handle<RMesh<B>>>,
    pub texture_handle: Option<Handle<RTexture<B>>>,
}

impl<B: hal::Backend> std::default::Default for TRenderable<B> {
    fn default() -> Self {
        TRenderable {
            mesh_handle: None,
            texture_handle: None,
        }
    }
}

impl<B: hal::Backend> TRenderable<B> {
    pub fn add_tex_renderable(world: &mut World, id: Entity, mesh: TRenderable<B>) {
        world.add_component(id, mesh);
    }
}


pub struct RenderingManager {
    pub hardware: Hw,
    renderer: Rd,
}

unsafe impl Send for RenderingManager{}
unsafe impl Sync for RenderingManager{}

pub struct RenderingSystem;

impl RenderingSystem {
    pub fn initialize(world: &mut World, res_path: &str) {
        let mut hardware = Hw::create();


        let asset_manager = world.resources.get::<AssetManager>().unwrap();
        let mesh_storage = AssetStorage::empty(&asset_manager, &["obj"]);
        let texture_storage = AssetStorage::empty(&asset_manager, &["png"]);
        drop(asset_manager);

        world.resources.insert::<AssetStorage<Mesh>>(mesh_storage);
        world.resources.insert::<AssetStorage<Texture>>(texture_storage);
        let renderer = Rd::create(&mut hardware, world);
        world
            .resources
            .insert::<RenderingManager>(RenderingManager { hardware, renderer });
    }

    pub fn maintain(world: &mut World) {
        let mut rm =
            <Write<RenderingManager>>::fetch(
                &mut world.resources,
            );
        let hardware = &mut rm.hardware;
        hardware.factory.maintain(&mut hardware.families);

    }

    pub fn run(world: &mut World) {
        let (mut rm, mut am, mut asm, mut ast) =
            <(Write<RenderingManager>, Write<AssetManager>, Write<AssetStorage<Mesh>>, Write<AssetStorage<Texture>>)>::fetch(
                &mut world.resources,
            );

        asm.process(&mut am, "obj",|data: &[u8]| {
            (Self::process_model(&mut rm, data, "lol"), false)
        });

        ast.process(&mut am, "png", |data: &[u8]| {
            (Self::process_texture(&mut rm, data, "lol"), false)

        });

        let RenderingManager{hardware, renderer} = &mut (*rm);
        renderer
            .run(hardware, &world);

    }
    pub fn load_from_obj(
        bytes: &[u8],
    ) -> Result<Vec<(MeshBuilder<'static>, Option<String>)>, failure::Error> {
        let string = std::str::from_utf8(bytes)?;
        let set = obj::parse(string).map_err(|e| {
            failure::format_err!(
                "Error during parsing obj-file at line '{}': {}",
                e.line_number,
                e.message
            )
        })?;
        Self::load_from_data(set)
    }

    fn load_from_data(
        obj_set: obj::ObjSet,
    ) -> Result<Vec<(MeshBuilder<'static>, Option<String>)>, failure::Error> {
        // Takes a list of objects that contain geometries that contain shapes that contain
        // vertex/texture/normal indices into the main list of vertices, and converts to
        // MeshBuilders with Position, Normal, TexCoord.
        let mut objects = vec![];

        for object in obj_set.objects {
            for geometry in &object.geometry {
                let mut builder = MeshBuilder::new();

                let mut indices = Vec::new();

                geometry.shapes.iter().for_each(|shape| {
                    if let obj::Primitive::Triangle(v1, v2, v3) = shape.primitive {
                        indices.push(v1);
                        indices.push(v2);
                        indices.push(v3);
                    }
                });
                // We can't use the vertices directly because we have per face normals and not per vertex normals in most obj files
                // TODO: Compress duplicates and return indices for indexbuffer.
                let positions = indices
                    .iter()
                    .map(|index| {
                        let vertex: obj::Normal = object.vertices[index.0];
                        Position([vertex.x as f32, vertex.y as f32, vertex.z as f32])
                    })
                    .collect::<Vec<_>>();

                let normals = indices
                    .iter()
                    .map(|index| {
                        index
                            .2
                            .map(|i| {
                                let normal: obj::Normal = object.normals[i];
                                Normal([normal.x as f32, normal.y as f32, normal.z as f32])
                            })
                            .unwrap_or(Normal([0.0, 0.0, 0.0]))
                    })
                    .collect::<Vec<_>>();

                let tex_coords = indices
                    .iter()
                    .map(|index| {
                        index
                            .1
                            .map(|i| {
                                let tvertex: obj::TVertex = object.tex_vertices[i];
                                TexCoord([tvertex.u as f32, tvertex.v as f32])
                            })
                            .unwrap_or(TexCoord([0.0, 0.0]))
                    })
                    .collect::<Vec<_>>();

                debug_assert!(&normals.len() == &positions.len());
                debug_assert!(&tex_coords.len() == &positions.len());

                let verts: Vec<_> = positions
                    .into_iter()
                    .zip(normals)
                    .zip(tex_coords)
                    .map(|x| PosNormTex {
                        position: (x.0).0,
                        normal: (x.0).1,
                        tex_coord: x.1,
                    })
                    .collect();

                // builder.set_indices(indices.iter().map(|i| i.0 as u16).collect::<Vec<u16>>());

                builder.add_vertices(verts);
                //builder.add_vertices(normals);
                //builder.add_vertices(tex_coords);

                // TODO: Add Material loading
                objects.push((builder, geometry.material_name.clone()))
            }
        }
        Ok(objects)
    }
    fn process_texture(manager: &mut RenderingManager, data: &[u8], filename: &str) -> Texture {
        let image_reader = std::io::Cursor::new(data);

        let texture_builder = image::load_from_image(
            image_reader,
            image::ImageTextureConfig {
                generate_mips: true,
                ..Default::default()
            },
        )
        .unwrap();

        let texture = texture_builder
            .build(
                rendy::factory::ImageState {
                    queue: manager.hardware.families.family(manager.hardware.used_family).queue(0).id(),
                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                    access: hal::image::Access::SHADER_READ,
                    layout: hal::image::Layout::ShaderReadOnlyOptimal,
                },
                &mut manager.hardware.factory,
            )
            .unwrap();

        let factory = &mut manager.hardware.factory;
        let layout = factory
            .create_descriptor_set_layout(vec![
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
            ])
            .unwrap();
        let descriptor_set = factory
            .create_descriptor_set(Escape::share(layout))
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

        Texture {
            texture,
            desc: Escape::unescape(descriptor_set),
        }
    }

    fn process_model(manager: &mut RenderingManager, data: &[u8], filename: &str) -> Mesh {
        let obj_builder = Self::load_from_obj(data);
        let mut obj_builder = match obj_builder {
            Ok(x) => x,
            Err(y) => {
                println!("Error: {:?}", y);
                panic!("bad model");
            }
        };
        if obj_builder.len() > 1 {
            println!("model {:?} contains more than one object", filename);
            panic!("bad model");
        }
        let model_builder = obj_builder.pop().unwrap();
        let qid = manager.hardware.families.family(manager.hardware.used_family).queue(0).id();
        let model = model_builder.0.build(qid, &manager.hardware.factory).unwrap();
        model
    }

    pub fn shut_down(world: &mut World) {
        world.resources.remove::<AssetStorage<Mesh>>();
        world.resources.remove::<AssetStorage<Texture>>();

        match world.resources.remove::<RenderingManager>() {
            None => (),
            Some(RenderingManager{mut hardware, mut renderer}) => {
                renderer.dispose(&mut hardware, &world);
                drop(renderer);
                drop(hardware);
            }
        }
    }
}

pub struct Renderer<B: hal::Backend> {
    graph: Option<rendy::graph::Graph<B, World>>,
    push_constants_block: Arc<octo_node::PushConstantsBlock>,
}

impl<B: hal::Backend> Renderer<B> {
    pub fn create(
        hardware: &mut Hardware<B>,
        world: &mut World,
    ) -> Self {
        world.resources.insert(CameraData {
            position: Vec3::zeros(),
            rotation: Quat::identity(),
        });
        let size = hardware
            .window
            .get_inner_size()
            .unwrap()
            .to_physical(hardware.window.get_hidpi_factor());
        world.resources.insert(ViewportData {
            width: size.width as f32,
            height: size.height as f32,
            ratio: (size.width / size.height) as f32,
            camera_lens_height: 10.0f32,
        });
        let (graph, block) = fill_render_graph(hardware, world);
        Self {
            graph: Some(graph),
            push_constants_block: block,
        }
    }
    pub fn run(&mut self, hardware: &mut Hardware<B>, world: &World) {
        match &mut self.graph {
            Some(x) => {
                let size = hardware
                    .window
                    .get_inner_size()
                    .unwrap()
                    .to_physical(hardware.window.get_hidpi_factor());
                self.push_constants_block.clear();
                self.push_constants_block
                    .fill(world, Vec2::new(size.width as f32, size.height as f32));
                x.run(&mut hardware.factory, &mut hardware.families, world);
            }
            None => (),
        }
    }

    pub fn dispose(&mut self, hardware: &mut Hardware<B>, world: &World) {
        match self.graph.take() {
            Some(x) => {
                x.dispose(&mut hardware.factory, world);
            }
            None => (),
        }
    }
}



pub type Hw = Hardware<rendy::vulkan::Backend>;
pub type Rd = Renderer<rendy::vulkan::Backend>;

pub fn fill_render_graph<'a, B: hal::Backend>(
    hardware: &mut Hardware<B>,
    world: &World
) -> (
    rendy::graph::Graph<B, World>,
    Arc<octo_node::PushConstantsBlock>,
) {
    let mut graph_builder = GraphBuilder::<B, World>::new();

    assert!(hardware.surface.is_some());

    let surface = hardware.surface.take().unwrap();

    let size = hardware
        .window
        .get_inner_size()
        .unwrap()
        .to_physical(hardware.window.get_hidpi_factor());
    let window_kind = hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1);

    // final image to be shown on screen
    let color = graph_builder.create_image(
        window_kind,
        1,
        hardware.factory.get_surface_format(&surface),
        Some(hal::command::ClearValue::Color([0.1, 0.1, 0.1, 1.0].into())),
    );

    let window_size = hal::image::Kind::D2(size.width as u32, size.height as u32, 1, 1);
    // deferred_pass producing base data
    let (deferred_pass, (position, normal, albedo), depth) = {
        let deferred_desc = deferred_node::DeferredNodeDesc::new();

        let gbuffer_size = window_size;

        let position = graph_builder.create_image(
            gbuffer_size,
            1,
            deferred_desc.position_format(),
            Some(hal::command::ClearValue::Color([0.0, 0.0, 0.0, 0.0].into())),
        );
        let normal = graph_builder.create_image(
            gbuffer_size,
            1,
            deferred_desc.normal_format(),
            Some(hal::command::ClearValue::Color([0.0, 0.0, 0.0, 0.0].into())),
        );
        let albedo = graph_builder.create_image(
            gbuffer_size,
            1,
            deferred_desc.albedo_format(),
            Some(hal::command::ClearValue::Color([0.0, 0.0, 0.0, 0.0].into())),
        );

        let depth = graph_builder.create_image(
            window_kind,
            1,
            hal::format::Format::D16Unorm,
            Some(hal::command::ClearValue::DepthStencil(
                hal::command::ClearDepthStencil(1.0, 0),
            )),
        );
        let deferred_pass = graph_builder.add_node(
            deferred_desc
                .builder()
                .into_subpass()
                .with_color(position)
                .with_color(normal)
                .with_color(albedo)
                .with_depth_stencil(depth)
                .into_pass(),
        );

        (deferred_pass, (position, normal, albedo), depth)
    };

    // loading renderer definition

    let f = std::fs::read_to_string("dev_app/renderer.octo_bin").unwrap();

    //let f = include_str!("../test_module.octo_bin");
    let octo_module: OctoModule = serde_json::from_str(&f).unwrap();

    // make sure that our renderer fits objects rendering
    assert!(octo_module.required_input.len() == 3);
    assert!(octo_module.required_input[0].1 == octo_runtime::ValueType::Vec4);
    assert!(octo_module.required_input[1].1 == octo_runtime::ValueType::Vec4);
    assert!(octo_module.required_input[2].1 == octo_runtime::ValueType::Vec4);

    for uni in &octo_module.uniform_block {
        println!(
            "loading push constant of type {:?} and name {:?}",
            uni.1, uni.0
        );
    }

    let uniform_block = Arc::new(octo_node::PushConstantsBlock::new(
        &octo_module.uniform_block,
    ));

    // create needed textures
    // size not used for now
    let textures: Vec<_> = octo_module
        .textures
        .iter()
        .map(|(id, t, _size)| {
            use octo_runtime::TextureType::*;
            let t = match t {
                Float => rendy::hal::format::Format::Rgba32Sfloat,
                Vec2 => rendy::hal::format::Format::Rgba32Sfloat,
                Vec3 => rendy::hal::format::Format::Rgba32Sfloat,
                Vec4 => rendy::hal::format::Format::Rgba32Sfloat,
            };

            //ignoring size for now
            let image = graph_builder.create_image(
                window_size,
                1,
                t,
                Some(hal::command::ClearValue::Color([0.0, 0.0, 0.0, 1.0].into())),
            );

            (id, t, image)
        })
        .collect();

    // create passes builders
    let mut passes = vec![];

    for (id, pass) in octo_module.passes.iter().enumerate() {
        // temporary check
        assert!(id == pass.id);

        let node_desc = octo_node::OctoNodeDesc {
            images: pass
                .input
                .iter()
                .map(|x| {
                    use octo_runtime::InputType::*;
                    match x {
                        ProvidedTexture(id) => match id {
                            0 => hal::format::Format::Rgba32Sfloat,
                            1 => hal::format::Format::Rgba32Sfloat,
                            2 => hal::format::Format::Rgba32Sfloat,
                            _ => unreachable!(),
                        },
                        PipelineTexture(id) => textures[*id].1,
                    }
                })
                .collect(),
            vertex_shader: std::cell::RefCell::new(octo_module.basic_vertex_spirv.clone()),
            fragment_shader: std::cell::RefCell::new(
                octo_module.fragment_shaders[&pass.shader].clone(),
            ),
            stage_name: "test_stage".to_owned(),
            stage_id: id,
            push_constants_size: octo_module.uniform_block_size,
            push_constants_block: uniform_block.clone(),
            view_size: (size.width as f64, size.height as f64),
            _phantom: Default::default(),
        };
        passes.push(node_desc.builder());
    }

    // assign needed textures
    let mut id = 0;
    for pass in &mut passes {
        let definition = &octo_module.passes[id];
        for tex in &definition.input {
            use octo_runtime::InputType::*;
            let texture = match tex {
                PipelineTexture(id) => textures[*id].2,
                ProvidedTexture(id) => match id {
                    0 => position,
                    1 => normal,
                    2 => albedo,
                    _ => unreachable!(),
                },
            };
            pass.add_image(texture);
        }

        id = id + 1;
    }

    let mut subpasses: Vec<_> = passes
        .drain(0..passes.len())
        .map(|x| x.into_subpass())
        .collect();

    let mut id = 0;
    for pass in &mut subpasses {
        let definition = &octo_module.passes[id];
        id = id + 1;
        use octo_runtime::OutputType::*;
        match &definition.output {
            Result => {
                pass.add_color(color);
            }
            Textures(ids) => {
                for id in ids {
                    pass.add_color(textures[*id].2);
                }
            }
        }
    }

    let mut nodes = vec![];

    /*let mut ui_node = UiNodeDesc {
        res: resources.clone(),
    }
    .builder()
    .into_subpass()
    .with_color(color)
    .with_depth_stencil(depth);
    */
    let mut id = 0;
    let mut present_builder = PresentNode::builder(&hardware.factory, surface, color);
    for mut pass in subpasses.drain(0..subpasses.len()) {
        let definition = &octo_module.passes[id];
        id = id + 1;
        if let Some(deps) = &definition.dependencies {
            for dependency in deps {
                pass.add_dependency(nodes[*dependency]);
            }
        } else {
            pass.add_dependency(deferred_pass);
        }

        let node_id = graph_builder.add_node(pass.into_pass());

        if definition.output == octo_runtime::OutputType::Result {
            //ui_node.add_dependency(node_id);
            present_builder.add_dependency(node_id);

        }
        nodes.push(node_id);
    }
    //let ui_node = graph_builder.add_node(ui_node.into_pass());


    let frames = present_builder.image_count();

    graph_builder.add_node(present_builder);
    (
        graph_builder
            .with_frames_in_flight(frames)
            .build(&mut hardware.factory, &mut hardware.families, world)
            .unwrap(),
        uniform_block,
    )
}
