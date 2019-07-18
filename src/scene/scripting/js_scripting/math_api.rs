use crate::scene::math::*;
use super::js;
use js::ContextGuard;

macro_rules! val {
        ($x:expr, $g:ident) => {
            match $x {
                None => return Result::Err(js::value::null($g)),
                Some(x) => x,
            }
        }
    }
impl super::JsScriptEngine {
    pub fn create_math_api(&mut self) {
        let guard = self.guard();
        let global = guard.global();
        let math = js::value::Object::new(&guard);
        global.set(&guard, js::Property::new(&guard, "Math"), math.clone());
        Self::create_vector_api(&guard, &math);
    }

    fn create_vector_api(guard: &ContextGuard, parent: &js::value::Object){
        let vector_prototype = js::value::Object::new(guard);
        {
            // define methods here

        }

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
    }
}