use serde::{Serialize, Deserialize};
use schemars::{schema_for, JsonSchema};
use ron::de::from_str;
use crate::glm;
use crate::math::*;
use crate::core::{TransformHierarchy, GameObject};
use crate::graphics::{CameraData, ViewportData};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Renderable {
    mesh: String,
    texture: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Object {
    name: String,
    position: Option<[f32; 3]>,
    renderable: Option<Renderable>,
    script: Option<String>,
    children: Option<Vec<Object>>,
    scale: Option<[f32; 3]>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Scene {
    name: String,
    camera_rotation: [f32; 3],
    viewport_height: f32,
    objects: Vec<Object>,
}

pub fn deserialize_scene(path: &str, engine: &mut crate::core::JsEngine)-> Result<(),String> {
    let data_string = std::fs::read_to_string(path).unwrap();

    let scene: Scene = match from_str(&data_string){
        Ok(x) => x,
        Err(e)=>{
            return Err(format!("Error reading scene file: {}", e));
        }
    };

    let camera_rot = glm::rotate_x(&glm::rotate_y(&glm::rotate_x(&glm::identity(), -scene.camera_rotation[0]), -scene.camera_rotation[1]), -scene.camera_rotation[2]);
    engine.world.resources.get_mut::<CameraData>().unwrap().rotation = glm::to_quat(&camera_rot);
    engine.world.resources.get_mut::<ViewportData>().unwrap().camera_lens_height = scene.viewport_height;

    println!("loading scene {}.", scene.name);
    for obj in scene.objects{
        spawn_object(obj, None, engine);
    }

    return Result::Ok(());

}

fn spawn_object(object: Object, parent: Option<legion::prelude::Entity>, engine: &mut crate::core::JsEngine) {
    println!("loading object {}.",object.name);
    let obj = engine.create_game_object();


    GameObject::set_name(&mut engine.world, obj, object.name);
    engine.set_parent(obj, parent).unwrap();

    object.position.map(|x| {
        TransformHierarchy::set_local_position(&mut engine.world, obj, Vec3::new(x[0], x[1], x[2]));
    });
    object.scale.map(|x| TransformHierarchy::set_local_scale(&mut engine.world, obj, Vec3::new(x[0], x[1], x[2])));
    object.renderable.map(|x| engine.add_renderable(obj, &x.mesh, Some(&x.texture)));
    object.script.map(|x| engine.add_script(obj, &x));


    if object.children.is_some(){
        for child in object.children.unwrap() {
            spawn_object(child, Some(obj), engine);
        }
    }
}

