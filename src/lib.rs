extern crate cfg_if;
extern crate wasm_bindgen;
#[macro_use]
extern crate lazy_static;

mod utils;
mod communication;

use wasm_bindgen::prelude::*;
use web_sys::console;
use std::sync::Mutex;

use communication::ConnectionState;

lazy_static! {
    static ref DATA: Mutex<ConnectionState> = Mutex::new(ConnectionState::new());
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

#[wasm_bindgen]
pub fn connect(address: &str)-> bool {
    let mut state = DATA.lock().unwrap();
    match state.connect("nice address") {
        Result::Ok(_)=> true,
        Result::Err(_) => false,
    }
}

#[wasm_bindgen]
pub fn close_connection() {
}