#![allow(dead_code)]

use just::*;
use just_js::JsEngineConfig;

fn main() {
    let engine_config = JsEngineConfig {
        source_root: "dev_app/scripts".to_string(),
    };
    let _window_config = 1i32;
    let _renderer_config = 2i32;
    let _resources = 3i32;

    let mut engine = core::JsEngine::new(&engine_config, &"dev_app/res");

    scene_serialization::deserialize_scene("dev_app/scene.ron", &mut engine).unwrap();

    engine.run();
}
