extern crate cfg_if;
extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;

mod utils;
mod communication;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use web_sys::console;
use std::sync::Mutex;

use communication::WebSocketHandler;

lazy_static! {
    static ref DATA: Mutex<WebSocketHandler> = Mutex::new(WebSocketHandler{lol: 2});
}

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn upgr() {
    let mut val = DATA.lock().unwrap();
    val.lol +=1;
    console::log_2(&"hey: ".into(), &val.lol.into());
}

#[wasm_bindgen(start)]
pub fn starter(){
    console::log_1(&"hey".into());
}
