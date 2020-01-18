use crate::math::*;
use super::js;
use js::ContextGuard;
use js::value::Value;

use super::api_helpers::*;

use crate::scripting::InternalTypes;
use chakracore::value::function::CallbackInfo;


macro_rules! unary_fun {
    ($g: ident) => {
        fn $g(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value>{
            debug_assert!(info.arguments.len()==1);
            let arg = double!(guard, info.arguments[0]);

            return Result::Ok(make_double!(guard, arg.$g()).into());
        }
    };
}

unary_fun!(sqrt);
unary_fun!(sin);
unary_fun!(cos);

fn rand(guard: &ContextGuard, info: CallbackInfo) ->Result<Value, Value> {
    let val: f64 = rand::random();

    Result::Ok(make_double!(guard, val).into())
}

// fn sin(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value>{
//     debug_assert!(info.arguments.len()==1);
//     let arg = double!(guard, info.arguments[0]);

//     return Result::Ok(make_double!(guard, arg.sin()).into());
// }

// fn cos(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value>{
//     debug_assert!(info.arguments.len()==1);
//     let arg = double!(guard, info.arguments[0]);

//     return Result::Ok(make_double!(guard, arg.cos()).into());
// }

fn vec3_get_x(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};

    Result::Ok(make_double!(guard, this.data[0] as f64).into())
}

fn vec3_get_y(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};

    Result::Ok(make_double!(guard, this.data[1] as f64).into())
}

fn vec3_get_z(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};

    Result::Ok(make_double!(guard, this.data[2] as f64).into())
}


fn vec3_set_x(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    debug_assert!(info.arguments.len() == 1);
    let val = double!(guard, info.arguments[0]);
    this.data[0] = val as f32;

    Result::Ok(js::value::null(guard))
}

fn vec3_set_y(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    debug_assert!(info.arguments.len() == 1);
    let val = double!(guard, info.arguments[0]);
    this.data[1] = val as f32;

    Result::Ok(js::value::null(guard))
}
fn vec3_set_z(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    debug_assert!(info.arguments.len() == 1);
    let val = double!(guard, info.arguments[0]);
    this.data[2] = val as f32;

    Result::Ok(js::value::null(guard))
}

fn vec3_clone(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {

    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    let ctx = guard.context();
    let prototypes = prototypes(&ctx);

    let obj = js::value::External::new(guard, Box::new(this.clone()));
    obj.set_prototype(guard, prototypes[&InternalTypes::Vec3].clone()).unwrap();

    Result::Ok(obj.into())
}

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
fn create_vec3_prototype(guard: &ContextGuard) -> js::value::Object {
    let obj = js::value::Object::new(guard);

    full_prop!(guard, obj, "x", vec3_get_x, vec3_set_x);
    full_prop!(guard, obj, "y", vec3_get_y, vec3_set_y);
    full_prop!(guard, obj, "z", vec3_get_z, vec3_set_z);

    add_function(guard, &obj, "clone", mf!(vec3_clone));

    obj
}

// fn quat_from_euler(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {

//     debug_assert!(info.arguments.len() == 3);
//     let x = double!(guard, info.arguments[0]);
//     let y = double!(guard, info.arguments[1]);
//     let z = double!(guard, info.arguments[2]);
//     let quat = nalgebra_glm::quat_angle_axis(nalgebra_glm::half_pi(), &Vec3::new());
//     Result::Err
// }

fn create_quat_prototype(guard: &ContextGuard) -> js::value::Object {
    let obj = js::value::Object::new(guard);

    obj
}

impl super::JsScriptEngine {
    pub fn create_math_api(&mut self) {
        let math = self.create_api_module("Math");
        let guard = self.guard();
        let vec3_prototype = Self::create_vector_api(&guard, &math);
        let quat_prototype = Self::create_quat_api(&guard, &math);

        add_function(&guard, &math, "Sin", mf!(sin));
        add_function(&guard, &math, "Cos", mf!(cos));
        add_function(&guard, &math, "Sqrt", mf!(sqrt));
        add_function(&guard, &math, "Random", mf!(rand));

        drop(guard);
        self.prototypes.0.insert(InternalTypes::Vec3, vec3_prototype);
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

    fn create_vector_api(guard: &ContextGuard, parent: &js::value::Object)-> js::value::Object{
        let vector_prototype = create_vec3_prototype(guard);

        let proto2 = vector_prototype.clone();

        let factory_function = js::value::Function::new(guard, Box::new(move |g, args|{
            let values = match args.arguments.len() {
                3 => {
                    [
                        double!(g, args.arguments[0]),
                        double!(g, args.arguments[1]),
                        double!(g, args.arguments[2]),
                    ]
                },
                0 => [0f64; 3],
                _ => return Result::Err(js::value::null(g))
            };
            let obj = js::value::External::new(g, Box::new(Vec3::new(values[0] as f32, values[1] as f32, values[2] as f32)));
            obj.set_prototype(g, vector_prototype.clone()).unwrap();
            Result::Ok(obj.into())

        }));
        parent.set(guard, js::Property::new(guard,"Vector"), factory_function);
        proto2
    }
}