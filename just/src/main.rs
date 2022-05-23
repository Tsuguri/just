#![allow(dead_code)]

use just::*;
use just_v8js::engine::JsEngineConfig;

fn main() {
    let engine_config = JsEngineConfig {
        source_root: "dev_app/scripts".to_string(),
        v8_args: vec![],
    };
    let _window_config = 1i32;
    let _renderer_config = 2i32;
    let _resources = 3i32;

    core::JsEngine::new(engine_config, &"dev_app/res").run();
}
