use just_assets::{AssetManager, AssetStorage};
use just_core::{
    ecs::prelude::World,
    math::{Quat, Vec3},
};

use crate::{CameraData, Mesh, Texture, ViewportData};

pub struct RendererState;

impl RendererState {
    pub(crate) fn initialize(world: &mut World) {
        world.resources.insert(CameraData {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
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

    pub(crate) fn strip_down(world: &mut World) {
        world.resources.remove::<AssetStorage<Mesh>>();
        world.resources.remove::<AssetStorage<Texture>>();
        world.resources.remove::<CameraData>();
        world.resources.remove::<ViewportData>();
    }
}
