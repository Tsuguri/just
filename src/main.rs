mod scene;
mod graphics;
mod input;


use scene::scripting::JsEngineConfig;


use nalgebra_glm as glm;
use crate::graphics::ResourceManager;

fn main() {
    let engine_config = JsEngineConfig { source_root: "".to_string() };
    let window_config = 1i32;
    let renderer_config = 2i32;
    let resources = 3i32;

    let mut scene = scene::JsEngine::new(&engine_config, &1i32, &"dev_app/res".to_string());
    let obj = scene.create_game_object();






    scene.run();


}
