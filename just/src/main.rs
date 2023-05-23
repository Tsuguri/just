#![allow(dead_code)]

use just::*;

fn main() {
    let _window_config = 1i32;
    let _renderer_config = 2i32;
    let _resources = 3i32;

    let mut engine = core::Engine::new(&"dev_app/res");

    scene_serialization::deserialize_scene("dev_app/scene.ron", &mut engine).unwrap();

    engine.run();
}
