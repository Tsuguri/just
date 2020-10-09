use crate::core::GameObject;
use just_core::{
    GameObjectData,
    math::Vec3,
    traits::scripting::{
        ScriptApiRegistry,
        function_params::World,
    },
};



pub struct WorldApi;

impl WorldApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let namespace = registry.register_namespace("World", None);

        registry.register_function("findByName", Some(&namespace), |args: (String, World)| {
            let objs = GameObject::find_by_name(&args.1, &args.0);
            objs.into_iter().map(|x| GameObjectData { id: x }).collect::<Vec<_>>()
        });

        registry.register_function("createGameObject", Some(&namespace), |mut args: World| {
            let obj = GameObject::create_empty(&mut args);
            GameObjectData { id: obj }
        });

        registry.register_function("setCameraPosition", Some(&namespace), |mut args: (World, Vec3)| {
            (*args.0).resources.get_mut::<just_wgpu::CameraData>().unwrap().position = args.1;
        });
    }
}
