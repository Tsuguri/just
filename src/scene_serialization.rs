use serde::{Serialize, Deserialize};
use ron::de::from_str;
use crate::math::*;
use crate::traits::{
    GameObjectId,
    World,

};

#[derive(Serialize, Deserialize)]
pub struct Renderable {
    mesh: String,
    texture: String,
}

#[derive(Serialize, Deserialize)]
pub struct Object {
    name: String,
    position: Option<Vec3>,
    renderable: Option<Renderable>,
    script: Option<String>,
    children: Option<Vec<Object>>,
    scale: Option<Vec3>,
}

#[derive(Serialize, Deserialize)]
pub struct Scene {
    name: String,
    objects: Vec<Object>,
}

pub fn deserialize_scene(path: &str, engine: &mut crate::core::JsEngine) {
    let data_string = std::fs::read_to_string(path).unwrap();

    let scene: Scene = match from_str(&data_string){
        Ok(x) => x,
        Err(e)=>{
            println!("Error reading scene file: {}", e);
            panic!();
        }
    };

    println!("loading scene {}.", scene.name);
    for obj in scene.objects{
        spawn_object(obj, None, engine);
    }


}

fn spawn_object(object: Object, parent: Option<GameObjectId>, engine: &mut crate::core::JsEngine) {
    println!("loading object {}.",object.name);
    let obj = engine.create_game_object();
    engine.world.set_name(obj, object.name);
    engine.set_parent(obj, parent);

    object.position.map(|x| {
        engine.world.set_local_position(obj, x);
    });
    object.scale.map(|x| engine.world.set_local_scale(obj, x));
    object.renderable.map(|x| engine.add_renderable(obj, &x.mesh));
    object.script.map(|x| engine.add_script(obj, &x));


    if object.children.is_some(){
        for child in object.children.unwrap() {
            spawn_object(child, Some(obj), engine);
        }
    }
}