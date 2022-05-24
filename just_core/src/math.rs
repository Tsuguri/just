use glam::Vec4Swizzles;

use crate::traits::scripting::{function_params::This, ScriptApiRegistry};
pub type Fl = f32;

pub type Vec2 = glam::Vec2;
pub type Vec3 = glam::Vec3;
pub type Vec4 = glam::Vec4;
pub type Matrix3 = glam::Mat3;
pub type Matrix = glam::Mat4;
pub type Quat = glam::Quat;

pub fn pos_vec(pos: &Vec3) -> Vec4 {
    pos.extend(1.0f32)
}

pub fn dir_vec(pos: &Vec3) -> Vec4 {
    pos.extend(0.0f32)
}

pub fn pos(vec: &Vec4) -> Vec3 {
    vec.xyz()
}

pub struct MathApi;

impl MathApi {
    pub fn register_api<'a, 'b, 'c, SAR: ScriptApiRegistry<'b, 'c>>(registry: &'a mut SAR) {
        let namespace = registry.register_namespace("Math", None);

        registry.register_function("Sin", Some(namespace), |args: f32| args.sin());
        registry.register_function("Cos", Some(namespace), |args: f32| args.cos());
        registry.register_function("Sqrt", Some(namespace), |args: f32| args.cos());
        registry.register_function("Sqrt", Some(namespace), |_: ()| rand::random::<f32>());

        registry
            .register_native_type("Vector2", Some(namespace), |args: (f32, f32)| Vec2::new(args.0, args.1))
            .unwrap();
        registry
            .register_native_type("Vector3", Some(namespace), |args: (f32, f32, f32)| {
                Vec3::new(args.0, args.1, args.2)
            })
            .unwrap();

        registry
            .register_native_type_method::<Vec3, _, _, _>("Clone", |args: This<Vec3>| *args.val)
            .unwrap();
        registry
            .register_native_type_method::<Vec3, _, _, _>("Len", |args: (This<Vec3>, Vec3)| args.0.distance(args.1))
            .unwrap();

        registry.register_native_type_property::<Vec3, _, _, _, _, _>(
            "x",
            Some(|args: This<Vec3>| args.val[0]),
            Some(|args: (This<Vec3>, f32)| args.0.val[0] = args.1),
        );
        registry.register_native_type_property::<Vec3, _, _, _, _, _>(
            "y",
            Some(|args: This<Vec3>| args.val[1]),
            Some(|args: (This<Vec3>, f32)| args.0.val[1] = args.1),
        );
        registry.register_native_type_property::<Vec3, _, _, _, _, _>(
            "z",
            Some(|args: This<Vec3>| args.val[2]),
            Some(|args: (This<Vec3>, f32)| args.0.val[2] = args.1),
        );

        registry.register_native_type_property::<Vec2, _, _, _, _, _>(
            "x",
            Some(|args: This<Vec2>| args.val[0]),
            Some(|args: (This<Vec2>, f32)| args.0.val[0] = args.1),
        );
        registry.register_native_type_property::<Vec2, _, _, _, _, _>(
            "y",
            Some(|args: This<Vec2>| args.val[1]),
            Some(|args: (This<Vec2>, f32)| args.0.val[1] = args.1),
        );
    }
}
