#![allow(dead_code)]

mod scene;
mod graphics;
mod input;


use scene::scripting::JsEngineConfig;


use nalgebra_glm as glm;

fn main() {
    let engine_config = JsEngineConfig { source_root: "".to_string() };
    let _window_config = 1i32;
    let _renderer_config = 2i32;
    let _resources = 3i32;

    let mut scene = scene::JsEngine::new(&engine_config, &1i32, &"dev_app/res".to_string());
    let _obj = scene.create_game_object();






    scene.run();


}
