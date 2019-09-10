use crate::math::*;
use super::js;
use js::ContextGuard;
use js::value::Value;
use js::value::function::CallbackInfo;

macro_rules! val {
        ($x:expr, $g:ident) => {
            match $x {
                None => return Result::Err(js::value::null($g)),
                Some(x) => x,
            }
        }
    }


fn console_log(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let context = guard.context();
    for arg in args.arguments {
        print!("{}", arg.to_string(guard));
    }
    print!("\n");
    Result::Ok(js::value::null(guard))
}



impl super::JsScriptEngine {
    pub fn create_console_api(&mut self) {
        let guard = self.guard();
        let global = guard.global();
        let console = js::value::Object::new(&guard);
        global.set(&guard, js::Property::new(&guard, "console"), console.clone());

        let fun = js::value::Function::new(&guard, Box::new(|a,b| console_log(a,b)));
        console.set(&guard, js::Property::new(&guard, "log"), fun);
    }
}
