use crate::scene::math::*;
use super::js;
use js::ContextGuard;
use js::value::Value;

use super::api_helpers::*;
use crate::scene::scripting::InternalTypes;
use chakracore::value::function::CallbackInfo;

macro_rules! val {
        ($x:expr, $g:ident) => {
            match $x {
                None => return Result::Err(js::value::null($g)),
                Some(x) => x,
            }
        }
    }


fn sin(guard: &ContextGuard, info: CallbackInfo) -> Result<Value, Value>{
    debug_assert!(info.arguments.len()==1);

    let arg = val!(info.arguments[0].clone().into_number(), guard).value_double();

    return Result::Ok(js::value::Number::from_double(guard, arg.sin()).into());

}

impl super::JsScriptEngine {
    pub fn create_math_api(&mut self) {
        let math = self.create_api_module("Math");
        let guard = self.guard();
        let global = guard.global();
        let math = js::value::Object::new(&guard);
        global.set(&guard, js::Property::new(&guard, "Math"), math.clone());
        let vec3_prototype = Self::create_vector_api(&guard, &math);

        let fun = js::value::Function::new(&guard, Box::new(|g,args| sin(g, args)));
        math.set(&guard, js::Property::new(&guard, "Sin"), fun);

        drop(guard);
        self.prototypes.0.insert(InternalTypes::Vec3, vec3_prototype);
    }

    fn create_vector_api(guard: &ContextGuard, parent: &js::value::Object)-> js::value::Object{
        let vector_prototype = js::value::Object::new(guard);
        {
            // define methods here

        }
        let proto2 = vector_prototype.clone();

        let factory_function = js::value::Function::new(guard, Box::new(move |g, args|{
            let values = match args.arguments.len() {
                3 => {
                    [
                        val!(args.arguments[0].clone().into_number(), g).value_double(),
                        val!(args.arguments[1].clone().into_number(), g).value_double(),
                        val!(args.arguments[2].clone().into_number(), g).value_double(),
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