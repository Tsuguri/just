#![allow(dead_code)]

use just_v8js::V8Engine;

fn main() {
    let mut engine = V8Engine::create(vec![]);
    println!("hello world");
    engine.run("lol(); 2+3");
    engine.lol();
}
