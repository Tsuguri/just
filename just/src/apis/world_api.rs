use crate::core::GameObject;
use just_core::{
    math::{Quat, Vec3},
    traits::scripting::{function_params::World, ScriptApiRegistry},
    GameObjectData,
};

pub struct WorldApi;

impl WorldApi {
    pub fn register_api<'a, 'b, 'c, SAR: ScriptApiRegistry<'b, 'c>>(registry: &'a mut SAR) {
        let namespace = registry.register_namespace("World", None);

        registry.register_function("findByName", Some(namespace), |args: (String, World)| {
            let objs = GameObject::find_by_name(&args.1, &args.0);
            objs.into_iter().map(|x| GameObjectData { id: x }).collect::<Vec<_>>()
        });

        registry.register_function("createGameObject", Some(namespace), |mut args: World| {
            let obj = GameObject::create_empty(&mut args);
            GameObjectData { id: obj }
        });

        registry.register_function("setCameraPosition", Some(namespace), |args: (World, Vec3)| {
            (*args.0)
                .resources
                .get_mut::<just_rend3d::CameraData>()
                .unwrap()
                .position = args.1;
        });

        registry.register_function("setCameraRotation", Some(namespace), |args: (World, Quat)| {
            (*args.0)
                .resources
                .get_mut::<just_rend3d::CameraData>()
                .unwrap()
                .rotation = args.1
        });
    }
}
