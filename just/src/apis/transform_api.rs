use just_core::traits::scripting::{
    ScriptApiRegistry,
    function_params::{This, World},
};


use just_core::GameObjectData;

use crate::core::GameObject;
use crate::core::TransformHierarchy;
use just_core::math::Vec3;

pub struct TransformApi {}

impl TransformApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let go_type = registry.get_native_type::<GameObjectData>().unwrap();

        registry.register_native_type_property(
            &go_type,
            "name",
            Some(|args: (This<GameObjectData>, World)| -> String {
                GameObject::get_name(&args.1, args.0.id)
            }),
            Some(|mut args: (This<GameObjectData>, World, String)| {
                GameObject::set_name(&mut args.1, args.0.id, args.2);
            }),
        );

        registry
            .register_native_type_method(
                &go_type,
                "destroy",
                |mut args: (This<GameObjectData>, World)| {
                    GameObject::delete(&mut args.1, args.0.id);
                },
            )
            .unwrap();

        registry.register_native_type_property(
            &go_type,
            "position",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_local_position(&args.1, args.0.id)
            }),
            Some(|mut args: (This<GameObjectData>, World, Vec3)| {
                TransformHierarchy::set_local_position(&mut args.1, args.0.id, args.2);
            }),
        );

        registry.register_native_type_property(
            &go_type,
            "globalPosition",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_global_position(&args.1, args.0.id)
            }),
            None::<fn(())>,
        );

        registry.register_native_type_property(
            &go_type,
            "scale",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_local_scale(&args.1, args.0.id)
            }),
            Some(|mut args: (This<GameObjectData>, World, Vec3)| {
                TransformHierarchy::set_local_scale(&mut args.1, args.0.id, args.2);
            }),
        );

        registry.register_native_type_property(
            &go_type,
            "parent",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_parent(&args.1, args.0.id).map(|x| GameObjectData { id: x })
            }),
            Some(
                |mut args: (This<GameObjectData>, World, Option<GameObjectData>)| {
                    TransformHierarchy::set_parent(&mut args.1, args.0.id, args.2.map(|x| x.id));
                },
            ),
        );

    }
}
