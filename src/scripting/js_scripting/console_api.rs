use crate::traits::ScriptApiRegistry;

pub struct ConsoleApi;

impl ConsoleApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let namespace = registry.register_namespace("console", None);

        registry.register_function("log", Some(&namespace), |args: Vec<String>| {
            for arg in args {
                print!("{}", arg);
            }
            print!("\n");
        });
    }
}
