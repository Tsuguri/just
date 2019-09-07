use super::js;
use crate::scene::WorldData;
use crate::scene::scripting::HM;

impl super::JsScriptEngine {
    pub fn create_api_module(&mut self, name: &str) -> js::value::Object {
        let guard = self.guard();
        let global = guard.global();
        let module = js::value::Object::new(&guard);
        global.set(&guard, js::Property::new(&guard, name), module.clone());
        module
    }
}

pub fn world(ctx: &js::Context) -> &mut WorldData{
   *ctx.get_user_data_mut::<&mut WorldData>().unwrap()
}

pub fn prototypes(ctx: &js::Context) -> &HM {
    *ctx.get_user_data::<&HM>().unwrap()
}