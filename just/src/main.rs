#![allow(dead_code)]

use just::*;

fn main() {
    let _window_config = 1i32;
    let _renderer_config = 2i32;
    let _resources = 3i32;

    core::Engine::new(&"dev_app/res").run();
}
