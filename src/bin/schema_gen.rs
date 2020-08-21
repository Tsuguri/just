#![allow(dead_code)]

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use just::scene_serialization::*;

fn generate_schemas() {
    use std::fs::File;
    use std::io::prelude::*;

    std::fs::create_dir_all("schemas");

    let scene_schema = schema_for!(Scene);
    let mut file = File::create("schemas/scene.schema").unwrap();
    file.write_all(
        serde_json::to_string_pretty(&scene_schema)
            .unwrap()
            .as_bytes(),
    )
    .unwrap();
}

fn main() {
    generate_schemas();
}
