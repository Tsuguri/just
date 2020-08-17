use crate::math::*;
use super::js;
use js::ContextGuard;
use js::value::Value;

use super::api_helpers::*;

use crate::scripting::InternalTypes;
use chakracore::value::function::CallbackInfo;

use crate::traits::*;

fn vec3_len(guard: &ContextGuard, info:CallbackInfo) ->Result<Value, Value> {
    let mut info = info;
    debug_assert!(info.arguments.len() == 2);
    let external2 = info.arguments.pop().unwrap().into_external().unwrap();
    let external1 = info.arguments.pop().unwrap().into_external().unwrap();
    let vec1 = unsafe {external1.value::<Vec3>()};
    let vec2 = unsafe {external2.value::<Vec3>()};

    let ctx = guard.context();
    let prototypes = prototypes(&ctx);
    let len = nalgebra_glm::distance(&vec1, &vec2);


    Result::Ok(make_double!(guard, len as f64).into())

}

fn create_quat_prototype(guard: &ContextGuard) -> js::value::Object {
    let obj = js::value::Object::new(guard);

    obj
}

pub struct MathAPI;

impl FunctionResult for Vec3 {}
impl FunctionParameter for Vec3 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_native()
    }
}
impl FunctionResult for Vec2 {}
impl FunctionParameter for Vec2 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_native()
    }
}

impl MathAPI {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let namespace = registry.register_namespace("Math2", None);

        registry.register_function("Sin", Some(&namespace), |args: f32| {
            args.sin()
        });
        registry.register_function("Cos", Some(&namespace), |args: f32| {
            args.cos()
        });
        registry.register_function("Sqrt", Some(&namespace), |args: f32| {
            args.cos()
        });
        registry.register_function("Sqrt", Some(&namespace), |_: ()| {
            rand::random::<f32>()
        });

        let vec2_type = registry.register_native_type("Vector2", Some(&namespace), |args: (f32, f32)|{
            Vec2::new(args.0, args.1)
        }).unwrap();
        let vec3_type = registry.register_native_type("Vector3", Some(&namespace), |args: (f32, f32, f32)|{
            Vec3::new(args.0, args.1, args.2)
        }).unwrap();

        registry.register_native_type_method::<Vec3, _, _, _>("Clone", |args: This<Vec3>| {*args.val}).unwrap();
        registry.register_native_type_method::<Vec3, _, _, _>("Len", |args: (This<Vec3>, Vec3)| {nalgebra_glm::distance(&*args.0, &args.1) as f32}).unwrap();

        registry.register_native_type_property::<Vec3,_, _, _, _, _>("x", Some(|args: This<Vec3>|{args.val[0]}), Some(|args: (This<Vec3>, f32)|{args.0.val[0] = args.1}));
        registry.register_native_type_property::<Vec3,_, _, _, _, _>("y", Some(|args: This<Vec3>|{args.val[1]}), Some(|args: (This<Vec3>, f32)|{args.0.val[1] = args.1}));
        registry.register_native_type_property::<Vec3,_, _, _, _, _>("z", Some(|args: This<Vec3>|{args.val[2]}), Some(|args: (This<Vec3>, f32)|{args.0.val[2] = args.1}));

        registry.register_native_type_property::<Vec2,_, _, _, _, _>("x", Some(|args: This<Vec2>|{args.val[0]}), Some(|args: (This<Vec2>, f32)|{args.0.val[0] = args.1}));
        registry.register_native_type_property::<Vec2,_, _, _, _, _>("y", Some(|args: This<Vec2>|{args.val[1]}), Some(|args: (This<Vec2>, f32)|{args.0.val[1] = args.1}));
    }
}

impl super::JsScriptEngine {
    pub fn create_math_api(&mut self) {
        let math = self.create_api_module("Math");
        let guard = self.guard();
        let quat_prototype = Self::create_quat_api(&guard, &math);

        drop(guard);
        self.prototypes.0.insert(InternalTypes::Quat, quat_prototype);
    }

    fn create_quat_api(guard: &ContextGuard, parent: &js::value::Object) -> js::value::Object {
        let quat_prototype = create_quat_prototype(guard);
        let quat2 = quat_prototype.clone();

        let factory_function = js::value::Function::new(guard, Box::new(move |g, args|{
            let obj = js::value::External::new(g, Box::<Quat>::new(Quat::identity()));
            obj.set_prototype(g, quat_prototype.clone()).unwrap();
            Result::Ok(obj.into())
        }));
        parent.set(guard, js::Property::new(guard, "Quat"), factory_function);
        quat2
    }

}
