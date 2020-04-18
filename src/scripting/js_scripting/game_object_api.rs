use crate::math::*;
use super::js;
use js::{
    ContextGuard,
    value::{
        Value,
        function::CallbackInfo,
    },
};

use super::api_helpers::*;
use super::ScriptCreationData;
use crate::scripting::InternalTypes;
use crate::scripting::js_scripting::JsScript;
use crate::scripting::js_scripting::resources_api::MeshData;
use legion::prelude::Entity;
use crate::core::{TransformHierarchy, GameObject};

pub struct GameObjectData {
    pub id: Entity,
}

fn get_name(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {

    let external = args.this.into_external().unwrap();
    let ctx = guard.context();
    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let name = GameObject::get_name(world.get_legion(), this.id);

    let val = js::value::String::new(guard, &name);

    Result::Ok(val.into())
}

fn set_name(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    debug_assert_eq!(args.arguments.len(), 1);
    let external = args.this.into_external().unwrap();
    let ctx = guard.context();
    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let new_name = args.arguments[0].to_string(guard);
    GameObject::set_name(world.get_legion(), this.id, new_name);

    Result::Ok(js::value::null(guard))
}

fn get_position(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let external = args.this.into_external().unwrap();

    let prototypes = prototypes(&ctx);


    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let pos = TransformHierarchy::get_local_position(world.get_legion(), this.id);

    let obj = js::value::External::new(guard, Box::new(pos));
    obj.set_prototype(guard, prototypes[&InternalTypes::Vec3].clone()).unwrap();

    Result::Ok(obj.into())
}

fn get_global_position(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let external = args.this.into_external().unwrap();

    let prototypes = prototypes(&ctx);

    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let pos = TransformHierarchy::get_global_position(world.get_legion(), this.id);

    let obj = js::value::External::new(guard, Box::new(pos));
    obj.set_prototype(guard, prototypes[&InternalTypes::Vec3].clone()).unwrap();

    Result::Ok(obj.into())

}

fn get_scale(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let external = args.this.into_external().unwrap();

    let prototypes = prototypes(&ctx);


    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let pos = TransformHierarchy::get_local_scale(world.get_legion(), this.id);

    let obj = js::value::External::new(guard, Box::new(pos));
    obj.set_prototype(guard, prototypes[&InternalTypes::Vec3].clone()).unwrap();

    Result::Ok(obj.into())

}

fn get_parent(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let external = args.this.into_external().unwrap();

    let prototypes = prototypes(&ctx);


    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let parent = TransformHierarchy::get_parent(world.get_legion(), this.id);
    match parent {
        Some(x) => {
            let obj = js::value::External::new(guard, Box::new(GameObjectData { id: x }));
            obj.set_prototype(guard, prototypes[&InternalTypes::GameObject].clone()).unwrap();

            Result::Ok(obj.into())
        }
        None => Result::Ok(js::value::null(guard)),
    }
}

fn set_parent(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    debug_assert_eq!(args.arguments.len(), 1);
    let ctx = guard.context();
    let external = args.this.into_external().unwrap();

    let world = world(&ctx);
    let this = unsafe { external.value::<GameObjectData>() };

    let new_parent = if args.arguments[0].is_null() {
        None
    } else {
        let arg = args.arguments[0].clone().into_external().unwrap();
        let par = unsafe{ arg.value::<GameObjectData>()};

        Some(par.id)
    };

    TransformHierarchy::set_parent(world.get_legion(), this.id, new_parent).unwrap();
    Result::Ok(js::value::null(guard))
}

fn set_position(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    debug_assert!(args.arguments.len() == 1);
    let np = args.arguments[0].clone().into_external().unwrap();
    let new_pos = unsafe { np.value::<Vec3>() };

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let ctx = guard.context();
    let world = world(&ctx);

    TransformHierarchy::set_local_position(world.get_legion(), this.id, *new_pos);

    Result::Ok(js::value::null(guard))
}

fn set_scale(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    debug_assert!(args.arguments.len() == 1);
    let np = args.arguments[0].clone().into_external().unwrap();
    let new_scale = unsafe { np.value::<Vec3>() };

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let ctx = guard.context();
    let world = world(&ctx);

    TransformHierarchy::set_local_scale(world.get_legion(), this.id, *new_scale);

    Result::Ok(js::value::null(guard))
}

fn set_renderable(guard: &ContextGuard, args: CallbackInfo)-> Result<Value, Value>{

    let ctx = guard.context();
    let world = world(&ctx);

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };

    let m = args.arguments[0].clone().into_external().unwrap();
    let mesh = unsafe{m.value::<MeshData>()};

    world.set_renderable(this.id, mesh.id);

    Result::Ok(js::value::null(guard))
}

fn set_script(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let creation_data = creation_data(&ctx);

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let m = args.arguments[0].clone().into_string().unwrap();

    creation_data.push(ScriptCreationData{object: this.id, script_type: m.value()});
    Result::Ok(js::value::null(guard))
}

fn get_script(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let script = world.get_legion().get_component::<JsScript>(this.id);
    match script {
        None => Result::Err(js::value::null(guard)),
        Some(x) => {
            Result::Ok(x.js_object.clone().into())
        }
    }
}

fn destroy(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };

    GameObject::delete(world.get_legion(), this.id);

    Result::Ok(js::value::null(guard))

}

fn test_go_function(guard: &ContextGuard, _args: CallbackInfo) -> Result<Value, Value> {
    Result::Ok(js::value::null(guard))
}


impl super::JsScriptEngine {
    pub fn create_external_prototype(guard: &ContextGuard) -> js::value::Object {
        let obj = js::value::Object::new(guard);

        add_function(guard, &obj, "test", mf!(test_go_function));

        full_prop!(guard, obj, "name", get_name, set_name);
        full_prop!(guard, obj, "position", get_position, set_position);
        full_prop!(guard, obj, "scale", get_scale, set_scale);
        full_prop!(guard, obj, "parent", get_parent, set_parent);

        let prop = js::value::Object::new(guard);
        getter!(guard, prop, get_global_position);
        obj.define_property(guard, js::Property::new(guard, "globalPosition"), prop);

        add_function(guard, &obj, "destroy", mf!(destroy));
        add_function(guard, &obj, "setRenderable", mf!(set_renderable));
        add_function(guard, &obj, "setScript", mf!(set_script));
        add_function(guard, &obj, "getScript", mf!(get_script));

        obj
    }

    pub fn create_script_external(&self, guard: &ContextGuard, id: Entity) -> js::value::Value {
        let obj = js::value::External::new(guard, Box::new(GameObjectData { id }));
        obj.set_prototype(guard, (self.prototypes[&InternalTypes::GameObject]).clone()).unwrap();
        obj.into()
    }
    pub fn create_game_object_api(&mut self) {
        let guard = self.guard();
        let ext_prototype = Self::create_external_prototype(&guard);
        drop(guard);

        self.prototypes.insert(InternalTypes::GameObject, ext_prototype);
    }
}
