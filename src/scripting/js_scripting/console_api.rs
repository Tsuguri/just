use super::js::{
    ContextGuard,
    value::{
        null,
        Value,
        function::CallbackInfo,
        Function,
        Object,
    },
    Property,

};

fn console_log(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    for arg in args.arguments {
        print!("{}", arg.to_string(guard));
    }
    print!("\n");
    Result::Ok(null(guard))
}

impl super::JsScriptEngine {
    pub fn create_console_api(&mut self) {
        let guard = self.guard();
        let global = guard.global();
        let console = Object::new(&guard);
        global.set(&guard, Property::new(&guard, "console"), console.clone());

        let fun = Function::new(&guard, Box::new(|a,b| console_log(a,b)));
        console.set(&guard, Property::new(&guard, "log"), fun);
    }
}
