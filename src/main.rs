#![allow(dead_code)]

mod math;
mod traits;
mod scripting;
mod core;
mod graphics;
mod input;


use scripting::JsEngineConfig;
use traits::World;


use nalgebra_glm as glm;

fn main() {
    let engine_config = JsEngineConfig { source_root: "dev_app/scripts".to_string() };
    let _window_config = 1i32;
    let _renderer_config = 2i32;
    let _resources = 3i32;

    let mut engine = core::JsEngine::new(&engine_config, &1i32, &"dev_app/res".to_string());
    let obj = engine.create_game_object();
    engine.world.set_local_position(obj, glm::vec3(0.0f32, 1.0, 2.0));
    engine.add_renderable(obj, "teapot3");

    let obj2 = engine.create_game_object();
    engine.set_parent(obj2, Some(obj));

    engine.world.set_local_position(obj2, glm::vec3(2.0f32, 0.0, 1.0));

    engine.add_renderable(obj2, "cow1");

    engine.world.set_name(obj2, "lolz".into());

    engine.add_script(obj, "test_script");
    engine.add_script(obj2, "test2");




    engine.run();


}
