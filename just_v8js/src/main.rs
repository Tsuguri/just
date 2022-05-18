#![allow(dead_code)]

use just_v8js::engine::JsEngineConfig;
use just_v8js::engine::V8Engine;

fn main() {
    let config = JsEngineConfig {
        source_root: "dev_app/scripts".to_string(),
        v8_args: vec![],
    };
    let mut engine = V8Engine::create_without_api(config);
    println!("hello world");
    engine.run("lol(); 2+3");
}
