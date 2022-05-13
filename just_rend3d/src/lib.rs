use rend3::*;
use std::sync::Arc;

use just_assets::{AssetManager, AssetStorage, Handle};
use just_core::ecs::prelude::*;
use just_core::ecs::world::World;
use just_core::hierarchy::TransformHierarchy;
use just_core::math::{Quat, Vec3};
use just_core::traits::scripting::ScriptApiRegistry;
use just_core::transform::Transform;
pub use winit;

use winit::event_loop::EventLoop;

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
pub struct Mesh {
    pub handle: rend3::types::MeshHandle,
}
pub struct Texture {}
pub struct RenderingSystem {}
#[derive(Default)]
pub struct Renderable {
    pub object_handle: Option<rend3::types::ObjectHandle>,
}

struct RenderingManager {
    renderer: Arc<rend3::Renderer>,
    window: winit::window::Window,
    surface: Arc<rend3::types::Surface>,
    object_handle: rend3::types::ObjectHandle,
    base_rendergraph: rend3_routine::base::BaseRenderGraph,
    pbr_routine: rend3_routine::pbr::PbrRoutine,
    tonemapping_routine: rend3_routine::tonemapping::TonemappingRoutine,
    directional_handle: rend3::types::DirectionalLightHandle,
    resolution: glam::UVec2,
}
fn vertex(pos: [f32; 3]) -> glam::Vec3 {
    glam::Vec3::from(pos)
}

fn create_mesh() -> rend3::types::Mesh {
    let vertex_positions = [
        // far side (0.0, 0.0, 1.0)
        vertex([-1.0, -1.0, 1.0]),
        vertex([1.0, -1.0, 1.0]),
        vertex([1.0, 1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0]),
        // near side (0.0, 0.0, -1.0)
        vertex([-1.0, 1.0, -1.0]),
        vertex([1.0, 1.0, -1.0]),
        vertex([1.0, -1.0, -1.0]),
        vertex([-1.0, -1.0, -1.0]),
        // right side (1.0, 0.0, 0.0)
        vertex([1.0, -1.0, -1.0]),
        vertex([1.0, 1.0, -1.0]),
        vertex([1.0, 1.0, 1.0]),
        vertex([1.0, -1.0, 1.0]),
        // left side (-1.0, 0.0, 0.0)
        vertex([-1.0, -1.0, 1.0]),
        vertex([-1.0, 1.0, 1.0]),
        vertex([-1.0, 1.0, -1.0]),
        vertex([-1.0, -1.0, -1.0]),
        // top (0.0, 1.0, 0.0)
        vertex([1.0, 1.0, -1.0]),
        vertex([-1.0, 1.0, -1.0]),
        vertex([-1.0, 1.0, 1.0]),
        vertex([1.0, 1.0, 1.0]),
        // bottom (0.0, -1.0, 0.0)
        vertex([1.0, -1.0, 1.0]),
        vertex([-1.0, -1.0, 1.0]),
        vertex([-1.0, -1.0, -1.0]),
        vertex([1.0, -1.0, -1.0]),
    ];

    let index_data: &[u32] = &[
        0, 1, 2, 2, 3, 0, // far
        4, 5, 6, 6, 7, 4, // near
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // top
        20, 21, 22, 22, 23, 20, // bottom
    ];

    rend3::types::MeshBuilder::new(vertex_positions.to_vec(), rend3::types::Handedness::Left)
        .with_indices(index_data.to_vec())
        .build()
        .unwrap()
}

impl RenderingSystem {
    pub fn initialize(world: &mut World, event_loop: &EventLoop<()>) {
        Self::initialize_state(world);
        let window = {
            let mut builder = winit::window::WindowBuilder::new();
            builder = builder.with_title("just");
            builder.build(&event_loop).expect("Could not build window")
        };
        let window_size = window.inner_size();
        // Create the Instance, Adapter, and Device. We can specify preferred backend,
        // device name, or rendering profile. In this case we let rend3 choose for us.
        let iad = pollster::block_on(rend3::create_iad(None, None, None, None)).unwrap();
        // The one line of unsafe needed. We just need to guarentee that the window
        // outlives the use of the surface.
        //
        // SAFETY: this surface _must_ not be used after the `window` dies. Both the
        // event loop and the renderer are owned by the `run` closure passed to winit,
        // so rendering work will stop after the window dies.
        let surface = Arc::new(unsafe { iad.instance.create_surface(&window) });
        // Get the preferred format for the surface.
        let format = surface.get_preferred_format(&iad.adapter).unwrap();
        // Configure the surface to be ready for rendering.
        rend3::configure_surface(
            &surface,
            &iad.device,
            format,
            glam::UVec2::new(window_size.width, window_size.height),
            rend3::types::PresentMode::Mailbox,
        );
        // Make us a renderer.
        let renderer = rend3::Renderer::new(
            iad,
            rend3::types::Handedness::Left,
            Some(window_size.width as f32 / window_size.height as f32),
        )
        .unwrap();
        // Create the base rendergraph.
        let base_rendergraph = rend3_routine::base::BaseRenderGraph::new(&renderer);

        let mut data_core = renderer.data_core.lock();
        let pbr_routine = rend3_routine::pbr::PbrRoutine::new(&renderer, &mut data_core, &base_rendergraph.interfaces);
        drop(data_core);
        let tonemapping_routine =
            rend3_routine::tonemapping::TonemappingRoutine::new(&renderer, &base_rendergraph.interfaces, format);

        // Create mesh and calculate smooth normals based on vertices
        let mesh = create_mesh();

        // Add mesh to renderer's world.
        //
        // All handles are refcounted, so we only need to hang onto the handle until we
        // make an object.
        let mesh_handle = renderer.add_mesh(mesh);

        // Add PBR material with all defaults except a single color.
        let material = rend3_routine::pbr::PbrMaterial {
            albedo: rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
            ..rend3_routine::pbr::PbrMaterial::default()
        };
        let material_handle = renderer.add_material(material);

        // Combine the mesh and the material with a location to give an object.
        let object = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(mesh_handle),
            material: material_handle,
            transform: glam::Mat4::IDENTITY,
        };
        // Creating an object will hold onto both the mesh and the material
        // even if they are deleted.
        //
        // We need to keep the object handle alive.
        let object_handle = renderer.add_object(object);

        let view_location = glam::Vec3::new(3.0, 3.0, -5.0);
        let view = glam::Mat4::from_euler(glam::EulerRot::XYZ, -0.55, 0.5, 0.0);
        let view = view * glam::Mat4::from_translation(-view_location);

        // Set camera's location
        renderer.set_camera_data(rend3::types::Camera {
            projection: rend3::types::CameraProjection::Perspective { vfov: 60.0, near: 0.1 },
            view,
        });

        // Create a single directional light
        //
        // We need to keep the directional light handle alive.
        let directional_handle = renderer.add_directional_light(rend3::types::DirectionalLight {
            color: glam::Vec3::ONE,
            intensity: 10.0,
            // Direction will be normalized
            direction: glam::Vec3::new(-1.0, -4.0, 2.0),
            distance: 400.0,
        });
        let resolution = glam::UVec2::new(window_size.width, window_size.height);

        world.resources.insert::<RenderingManager>(RenderingManager {
            renderer,
            window,
            surface,
            object_handle,
            base_rendergraph,
            pbr_routine,
            tonemapping_routine,
            resolution,
            directional_handle,
        });
    }
    pub fn maintain(world: &mut World) {}
    pub fn update(world: &mut World) {
        let (manager, mut asset_manager, mut texture_storage, mut mesh_storage, camera_data, viewport_data) =
            <(
                Write<RenderingManager>,
                Write<AssetManager>,
                Write<AssetStorage<Texture>>,
                Write<AssetStorage<Mesh>>,
                Read<CameraData>,
                Read<ViewportData>,
            )>::fetch(&mut world.resources);
        texture_storage.process(&mut asset_manager, "png", |data| {
            (Self::load_png_texture(&manager, data), false)
        });

        mesh_storage.process(&mut asset_manager, "obj", |data| {
            (Self::load_obj_model(&manager, data), false)
        });

        // update objects transforms
        let query = <Read<Renderable>>::query();
        for (id, renderable) in query.iter_entities_immutable(world) {
            let gm = TransformHierarchy::get_global_matrix(world, id);
            let global_matrix = glam::Mat4::from_cols(
                glam::Vec4::new(gm.m11, gm.m12, gm.m13, gm.m14),
                glam::Vec4::new(gm.m21, gm.m22, gm.m23, gm.m24),
                glam::Vec4::new(gm.m31, gm.m32, gm.m33, gm.m34),
                glam::Vec4::new(gm.m41, gm.m42, gm.m43, gm.m44),
            )
            .transpose();
            manager
                .renderer
                .set_object_transform(renderable.object_handle.as_ref().unwrap(), global_matrix);
        }

        let frame = rend3::util::output::OutputFrame::Surface {
            surface: Arc::clone(&manager.surface),
        };
        // Ready up the renderer
        let (cmd_bufs, ready) = manager.renderer.ready();

        // Build a rendergraph
        let mut graph = rend3::graph::RenderGraph::new();

        // Add the default rendergraph without a skybox
        manager.base_rendergraph.add_to_graph(
            &mut graph,
            &ready,
            &manager.pbr_routine,
            None,
            &manager.tonemapping_routine,
            manager.resolution,
            rend3::types::SampleCount::One,
            glam::Vec4::ZERO,
            glam::Vec4::new(0.10, 0.05, 0.10, 1.0), // Nice scene-referred purple
        );

        // Dispatch a render using the built up rendergraph!
        graph.execute(&manager.renderer, frame, cmd_bufs, &ready);
    }
    pub fn register_api<'a, 'b, 'c, SAR: ScriptApiRegistry<'b, 'c>>(registry: &'a mut SAR) {}
    pub fn shut_down(world: &mut World) {
        let manago = world.resources.remove::<RenderingManager>();
        if let Some(manager) = manago {
            let RenderingManager {
                renderer,
                window,
                surface,
                object_handle,
                base_rendergraph,
                pbr_routine,
                tonemapping_routine,
                ..
            } = manager;
            drop(tonemapping_routine);
            drop(pbr_routine);
            drop(base_rendergraph);
            drop(object_handle);
            drop(renderer);
            drop(surface);
            drop(window);
        }
    }

    /// Only to be used in scene deserialization
    /// Does not check if renderable already exists
    pub fn add_renderable(world: &mut World, id: Entity, mesh: &str, texture: Option<&str>) {
        let position = TransformHierarchy::get_global_position(world, id);
        let manager = world.resources.get::<RenderingManager>().unwrap();
        let res2 = world.resources.get::<AssetStorage<Mesh>>().unwrap();
        let res = world.resources.get::<AssetStorage<Texture>>().unwrap();
        let mesh_handle = res2.get_handle(mesh).unwrap();
        let albedo_component = match texture {
            None => rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0)),
            Some(name) => {
                let tex_res = res.get_handle(name).unwrap();
                rend3_routine::pbr::AlbedoComponent::Value(glam::Vec4::new(0.0, 0.5, 0.5, 1.0))
                //rend3_routine::pbr::AlbedoComponent::Texture(tex_handle)
            }
        };
        let material = rend3_routine::pbr::PbrMaterial {
            albedo: albedo_component,
            ..rend3_routine::pbr::PbrMaterial::default()
        };
        let material_handle = manager.renderer.add_material(material);
        let object = rend3::types::Object {
            mesh_kind: rend3::types::ObjectMeshKind::Static(res2.get_value(&mesh_handle).unwrap().handle.clone()),
            material: material_handle,
            transform: glam::Mat4::IDENTITY,
        };
        let renderable = manager.renderer.add_object(object);
        drop(manager);
        drop(res2);
        drop(res);
        world.add_component(
            id,
            Renderable {
                object_handle: Some(renderable),
            },
        );
    }
}

impl RenderingSystem {
    fn update_positions(world: &mut World) {}
    fn load_png_texture(renderer: &RenderingManager, data: &[u8]) -> Texture {
        Texture {}
    }
    fn load_obj_model(renderer: &RenderingManager, data: &[u8]) -> Mesh {
        let mut cursor = std::io::Cursor::new(data);
        let (mut objects, materials) =
            tobj::load_obj_buf(&mut cursor, true, |arg| Result::Err(tobj::LoadError::ReadError)).unwrap();

        let first_mesh = objects.pop().unwrap();
        println!("processing mesh {}", first_mesh.name);
        let mut first_mesh = first_mesh.mesh;

        let num_vertices = first_mesh.positions.len() / 3;
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut uvs = Vec::with_capacity(num_vertices);

        if first_mesh.normals.is_empty() {
            first_mesh.normals.resize(num_vertices * 3, 0.5);
            println!(
                "setting normals with {}, results in {} normals",
                num_vertices * 3,
                first_mesh.normals.len()
            );
        }
        if first_mesh.texcoords.is_empty() {
            first_mesh.texcoords.resize(num_vertices * 2, 0.5);
            println!(
                "setting uvs with {}, results in {} uvs",
                num_vertices * 2,
                first_mesh.texcoords.len()
            );
        }
        for i in 0..first_mesh.positions.len() / 3 {
            positions.push(glam::Vec3::new(
                first_mesh.positions[i * 3],
                first_mesh.positions[i * 3 + 1],
                first_mesh.positions[i * 3 + 2],
            ));
            normals.push(glam::Vec3::new(
                first_mesh.normals[i * 3],
                first_mesh.normals[i * 3 + 1],
                first_mesh.normals[i * 3 + 2],
            ));
            uvs.push(glam::Vec2::new(
                first_mesh.texcoords[i * 2],
                first_mesh.texcoords[i * 2 + 1],
            ));
        }

        let mesh = rend3::types::MeshBuilder::new(positions, rend3::types::Handedness::Right)
            .with_vertex_normals(normals)
            .with_vertex_uv0(uvs)
            .with_indices(first_mesh.indices)
            .build()
            .unwrap();
        Mesh {
            handle: renderer.renderer.add_mesh(mesh),
        }
    }
    fn initialize_state(world: &mut World) {
        world.resources.insert(CameraData {
            position: Vec3::zeros(),
            rotation: Quat::identity(),
        });
        world.resources.insert(ViewportData {
            width: 0.0f32,
            height: 0.0f32,
            ratio: 1.0f32,
            camera_lens_height: 10.0f32,
        });
        let asset_manager = world.resources.get::<AssetManager>().unwrap();
        let mesh_storage = AssetStorage::empty(&asset_manager, &["obj"]);
        let texture_storage = AssetStorage::empty(&asset_manager, &["png"]);
        drop(asset_manager);

        world.resources.insert::<AssetStorage<Mesh>>(mesh_storage);
        world.resources.insert::<AssetStorage<Texture>>(texture_storage);
    }
}
