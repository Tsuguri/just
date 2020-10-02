use just_core::{
    traits::scripting::ScriptApiRegistry,
    ecs::prelude::*
};
use std::any::TypeId;
use std::collections::HashMap;

pub struct AssetManager {
    resources_path: String
}

pub struct AssetSystem;

impl AssetSystem {
    pub fn initialize(world: &mut World, resources: &str) {
        println!("initializing resource system");
        println!(
            "Asset system: Loading resources from: {}",
            std::fs::canonicalize(resources).unwrap().display()
        );
        world.resources.insert::<AssetManager>(AssetManager {
            resources_path: resources.to_owned(),
        });
    }

    pub fn register_api<SAR: ScriptApiRegistry>(sar: &mut SAR) {

    }

    pub fn update(world: &mut World) {

    }
}

pub struct AssetStorage<A> {
    assets: HashMap<u32, A>,
}

pub enum AssetState {
    Offline,
    Loading(f32), //progress here?
    Queued,
    Loaded,
}

pub struct Asset<A> {
    path: String,
    file_content: Option<Vec<u32>>,
    asset_content: Option<A>,
    state: AssetState,
}

