use crate::math::*;
use super::js;
use js::ContextGuard;
use js::value::Value;

use super::api_helpers::*;

use crate::scripting::InternalTypes;
use chakracore::value::function::CallbackInfo;

fn sin(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value>{
    debug_assert!(info.arguments.len()==1);
    let arg = double!(guard, info.arguments[0]);

    return Result::Ok(make_double!(guard, arg.sin()).into());
}

fn cos(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value>{
    debug_assert!(info.arguments.len()==1);
    let arg = double!(guard, info.arguments[0]);

    return Result::Ok(make_double!(guard, arg.cos()).into());
}

fn vec3_get_x(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};

    Result::Ok(make_double!(guard, this[0] as f64).into())
}

fn vec3_get_y(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};

    Result::Ok(make_double!(guard, this[1] as f64).into())
}

fn vec3_get_z(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};

    Result::Ok(make_double!(guard, this[2] as f64).into())
}


fn vec3_set_x(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    debug_assert!(info.arguments.len() == 1);
    let val = double!(guard, info.arguments[0]);
    this[0] = val as f32;

    Result::Ok(js::value::null(guard))
}

fn vec3_set_y(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    debug_assert!(info.arguments.len() == 1);
    let val = double!(guard, info.arguments[0]);
    this[1] = val as f32;

    Result::Ok(js::value::null(guard))
}
fn vec3_set_z(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value> {
    let external = info.this.into_external().unwrap();
    let this = unsafe {external.value::<Vec3>()};
    debug_assert!(info.arguments.len() == 1);
    let val = double!(guard, info.arguments[0]);
    this[2] = val as f32;

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
fn create_vec3_prototype(guard: &ContextGuard) -> js::value::Object {
    let obj = js::value::Object::new(guard);

    full_prop!(guard, obj, "x", vec3_get_x, vec3_set_x);
    full_prop!(guard, obj, "y", vec3_get_y, vec3_set_y);
    full_prop!(guard, obj, "z", vec3_get_z, vec3_set_z);

    add_function(guard, &obj, "clone", mf!(vec3_clone));

    obj
}

impl super::JsScriptEngine {
    pub fn create_math_api(&mut self) {
        let math = self.create_api_module("Math");
        let guard = self.guard();
        let vec3_prototype = Self::create_vector_api(&guard, &math);

        add_function(&guard, &math, "Sin", mf!(sin));

        drop(guard);
        self.prototypes.0.insert(InternalTypes::Vec3, vec3_prototype);
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