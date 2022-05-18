pub use crate::glm;
pub type Fl = f32;

pub type Vec2 = glm::TVec2<Fl>;
pub type Vec3 = glm::TVec3<Fl>;
pub type Vec4 = glm::TVec4<Fl>;
pub type Matrix3 = glm::TMat3<Fl>;
pub type Matrix = glm::TMat4<Fl>;
pub type Quat = glm::Qua<Fl>;

pub fn pos_vec(pos: &Vec3) -> Vec4 {
    Vec4::new(pos.data[0], pos.data[1], pos.data[2], 1.0f32)
}

pub fn dir_vec(pos: &Vec3) -> Vec4 {
    Vec4::new(pos.data[0], pos.data[1], pos.data[2], 0.0f32)
}

pub fn pos(vec: &Vec4) -> Vec3 {
    glm::vec4_to_vec3(vec)
}

pub struct MathApi;

impl MathApi {
    // pub fn register_api<'a, 'b, 'c, SAR: ScriptApiRegistry<'b, 'c>>(registry: &'a mut SAR) {
    //     let namespace = registry.register_namespace("Math", None);

    // registry.register_function("Sin", Some(namespace), |args: f32| args.sin());
    // registry.register_function("Cos", Some(namespace), |args: f32| args.cos());
    // registry.register_function("Sqrt", Some(namespace), |args: f32| args.cos());
    // registry.register_function("Sqrt", Some(namespace), |_: ()| rand::random::<f32>());

    // let vec2_type = registry
    //     .register_native_type("Vector2", Some(namespace), |args: (f32, f32)| Vec2::new(args.0, args.1))
    //     .unwrap();
    // let vec3_type = registry
    //     .register_native_type("Vector3", Some(namespace), |args: (f32, f32, f32)| {
    //         Vec3::new(args.0, args.1, args.2)
    //     })
    //     .unwrap();

    // registry
    //     .register_native_type_method(vec3_type, "Clone", |args: This<Vec3>| *args.val)
    //     .unwrap();
    // registry
    //     .register_native_type_method(vec3_type, "Len", |args: (This<Vec3>, Vec3)| {
    //         glm::distance(&*args.0, &args.1) as f32
    //     })
    //     .unwrap();

    // registry.register_native_type_property(
    //     vec3_type,
    //     "x",
    //     Some(|args: This<Vec3>| args.val[0]),
    //     Some(|args: (This<Vec3>, f32)| args.0.val[0] = args.1),
    // );
    // registry.register_native_type_property(
    //     vec3_type,
    //     "y",
    //     Some(|args: This<Vec3>| args.val[1]),
    //     Some(|args: (This<Vec3>, f32)| args.0.val[1] = args.1),
    // );
    // registry.register_native_type_property(
    //     vec3_type,
    //     "z",
    //     Some(|args: This<Vec3>| args.val[2]),
    //     Some(|args: (This<Vec3>, f32)| args.0.val[2] = args.1),
    // );

    // registry.register_native_type_property(
    //     vec2_type,
    //     "x",
    //     Some(|args: This<Vec2>| args.val[0]),
    //     Some(|args: (This<Vec2>, f32)| args.0.val[0] = args.1),
    // );
    // registry.register_native_type_property(
    //     vec2_type,
    //     "y",
    //     Some(|args: This<Vec2>| args.val[1]),
    //     Some(|args: (This<Vec2>, f32)| args.0.val[1] = args.1),
    // );
    // }
}
