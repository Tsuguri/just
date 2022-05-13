use just_core::traits::scripting::ScriptApiRegistry;

pub struct ConsoleApi;

impl ConsoleApi {
    pub fn register_api<'a, 'b, 'c, SAR: ScriptApiRegistry<'b, 'c>>(registry: &'a mut SAR) {
        let namespace = registry.register_namespace("console", None);

        registry.register_function("log", Some(namespace), |args: Vec<String>| {
            for arg in args {
                print!("{}", arg);
            }
            print!("\n");
        });
    }
}
