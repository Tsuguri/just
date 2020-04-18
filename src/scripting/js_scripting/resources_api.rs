use std::sync::Arc;
use super::js;
use js::{
    ContextGuard,
    value::{
        Value,
        function::CallbackInfo,
    },
};

use super::api_helpers::*;

use crate::traits::{
    TextureId,
    MeshId,
    ResourceProvider,
};

pub struct MeshData {
    pub id: MeshId,
}

pub struct TextureData {
    pub id: TextureId,
}


fn get_mesh(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let resources = world.get_legion().resources.get::<Arc<dyn ResourceProvider>>().unwrap();

    let name = args.arguments[0].to_string(guard);

    let res = resources.get_mesh(&name);

    match res {
        None =>{
            Result::Ok(js::value::null(guard))
        },
        Some(x)=>{
            Result::Ok(js::value::External::new(guard, Box::new(MeshData{id: x})).into())
        }
    }


}

impl super::JsScriptEngine {
    pub fn create_resources_api(&mut self){
        let module = self.create_api_module("Resources");
        let guard = self.guard();

        add_function(&guard, &module, "getMesh", mf!(get_mesh));

    }

}
