mod ui_node;

use just_core::ecs::prelude::*;
use just_core::math::{Quat, Vec2, Vec3};
use just_rendyocto::octo_runtime;
use just_rendyocto::rendy;

use crate::traits;

use just_rendyocto::resources::ResourceProvider;
use just_rendyocto::deferred_node;
use just_rendyocto::octo_node;
use just_rendyocto::CameraData;
use just_rendyocto::ViewportData;

#[cfg(test)]
pub mod test_resources;

pub use just_rendyocto::resources::ResourceManager;

use rendy::{
    command::Families,
    factory::{Config, Factory},
    graph::{present::PresentNode, render::*, GraphBuilder},
    hal,
    wsi::winit::{EventsLoop, Window, WindowBuilder},
};
use std::mem::ManuallyDrop;
use std::sync::Arc;

use octo_runtime::OctoModule;
use ui_node::UiNodeDesc;


pub struct RenderingManager {
    pub hardware: Hw,
    renderer: Rd,
    resources: Arc<dyn ResourceProvider>,
}

unsafe impl Send for RenderingManager{}
unsafe impl Sync for RenderingManager{}

pub struct RenderingSystem;

impl RenderingSystem {
    pub fn initialize(world: &mut World, res_path: &str) -> Arc<dyn ResourceProvider>{
        let mut hardware = Hw::create();
        let resources = Arc::new(Res::create(res_path, &mut hardware));
        // render graph elements are fetching stuff from resources Arc
        let renderer = Rd::create(&mut hardware, world, resources.clone());

        world
            .resources
            .insert::<Arc<dyn ResourceProvider>>(resources.clone());

        world
            .resources
            .insert::<RenderingManager>(RenderingManager { hardware, renderer, resources: resources.clone()});
        resources
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
        let mut rm =
            <Write<RenderingManager>>::fetch(
                &mut world.resources,
            );

        let RenderingManager{hardware, renderer, resources: _} = &mut (*rm);
        renderer
            .run(hardware, &world);

    }

    pub fn shut_down(world: &mut World) {
        world.resources.remove::<Arc<dyn ResourceProvider>>();

        match world.resources.remove::<RenderingManager>() {
            None => (),
            Some(RenderingManager{mut hardware, mut renderer, mut resources}) => {
                drop(resources);
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
        res: Arc<ResourceManager<B>>,
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
        let (graph, block) = fill_render_graph(hardware, world, res);
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

use just_rendyocto::Hardware;

pub type Hw = Hardware<rendy::vulkan::Backend>;
pub type Rd = Renderer<rendy::vulkan::Backend>;
pub type Res = just_rendyocto::resources::ResourceManager<rendy::vulkan::Backend>;

pub fn fill_render_graph<'a, B: hal::Backend>(
    hardware: &mut Hardware<B>,
    world: &World,
    resources: Arc<ResourceManager<B>>,
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
        let deferred_desc = deferred_node::DeferredNodeDesc {
            res: resources.clone(),
        };

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
            res: resources.clone(),
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

    let mut ui_node = UiNodeDesc {
        res: resources.clone(),
    }
    .builder()
    .into_subpass()
    .with_color(color)
    .with_depth_stencil(depth);
    let mut id = 0;
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
            ui_node.add_dependency(node_id);
        }
        nodes.push(node_id);
    }
    let ui_node = graph_builder.add_node(ui_node.into_pass());

    let mut present_builder = PresentNode::builder(&hardware.factory, surface, color);
    present_builder.add_dependency(ui_node);

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
