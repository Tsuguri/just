use super::js;
use crate::math::*;
use js::{
    value::{function::CallbackInfo, Value},
    ContextGuard,
};

use super::api_helpers::*;
use super::{ScriptCreationData, ScriptCreationQueue};
use crate::core::{GameObject, Mesh, TransformHierarchy};
use crate::scripting::js_scripting::resources_api::MeshData;
use crate::scripting::js_scripting::JsScript;
use crate::scripting::InternalTypes;
use crate::traits::*;
use legion::prelude::Entity;

#[derive(Debug, Copy, Clone)]
pub struct GameObjectData {
    pub id: Entity,
}

#[derive(Debug, Copy, Clone)]
pub struct ComponentData<T> {
    pub id: Entity,
    _phantom: std::marker::PhantomData<T>,
}

impl FunctionResult for GameObjectData {}
impl FunctionParameter for GameObjectData {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

fn set_renderable(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };

    let m = args.arguments[0].clone().into_external().unwrap();
    let mesh = unsafe { m.value::<MeshData>() };

    Mesh::add_renderable_to_go(world, this.id, mesh.id);

    Result::Ok(js::value::null(guard))
}

fn set_script(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let creation_data = &mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q;

    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let m = args.arguments[0].clone().into_string().unwrap();

    creation_data.push(ScriptCreationData {
        object: this.id,
        script_type: m.value(),
    });
    Result::Ok(js::value::null(guard))
}

fn get_script(guard: &ContextGuard, args: CallbackInfo) -> Result<Value, Value> {
    let ctx = guard.context();
    let world = world(&ctx);
    let te = args.this.into_external().unwrap();
    let this = unsafe { te.value::<GameObjectData>() };
    let script = world.get_component::<JsScript>(this.id);
    match script {
        None => Result::Err(js::value::null(guard)),
        Some(x) => Result::Ok(x.js_object.clone().into()),
    }
}

pub struct GameObjectApi {}

impl GameObjectApi {
    pub fn register<SAR: ScriptApiRegistry>(registry: &mut SAR) {
        let go_type = registry
            .register_native_type("GameObject", None, |arg: GameObjectData| arg)
            .unwrap();

        registry.register_native_type_property(
            &go_type,
            "name",
            Some(|args: (This<GameObjectData>, World)| -> String {
                GameObject::get_name(&args.1, args.0.id)
            }),
            Some(|mut args: (This<GameObjectData>, World, String)| {
                GameObject::set_name(&mut args.1, args.0.id, args.2);
            }),
        );

        registry.register_native_type_property(
            &go_type,
            "position",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_local_position(&args.1, args.0.id)
            }),
            Some(|mut args: (This<GameObjectData>, World, Vec3)| {
                TransformHierarchy::set_local_position(&mut args.1, args.0.id, args.2);
            }),
        );

        registry.register_native_type_property(
            &go_type,
            "globalPosition",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_global_position(&args.1, args.0.id)
            }),
            Some(|()| {}),
        );

        registry.register_native_type_property(
            &go_type,
            "scale",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_local_scale(&args.1, args.0.id)
            }),
            Some(|mut args: (This<GameObjectData>, World, Vec3)| {
                TransformHierarchy::set_local_scale(&mut args.1, args.0.id, args.2);
            }),
        );

        registry.register_native_type_property(
            &go_type,
            "parent",
            Some(|args: (This<GameObjectData>, World)| {
                TransformHierarchy::get_parent(&args.1, args.0.id).map(|x| GameObjectData { id: x })
            }),
            Some(
                |mut args: (This<GameObjectData>, World, Option<GameObjectData>)| {
                    TransformHierarchy::set_parent(&mut args.1, args.0.id, args.2.map(|x| x.id));
                },
            ),
        );

        registry
            .register_native_type_method(
                &go_type,
                "destroy",
                |mut args: (This<GameObjectData>, World)| {
                    GameObject::delete(&mut args.1, args.0.id);
                },
            )
            .unwrap();
    }
}

impl super::JsScriptEngine {
    pub fn create_go_external(&self, guard: &ContextGuard, id: Entity) -> js::value::Value {
        let obj = js::value::External::new(guard, Box::new(GameObjectData { id }));
        let type_id = std::any::TypeId::of::<GameObjectData>();
        obj.set_prototype(guard, (self.external_types_prototypes[&type_id]).clone())
            .unwrap();

        obj.into()
    }
}
