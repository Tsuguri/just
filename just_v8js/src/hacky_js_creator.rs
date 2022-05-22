use std::convert::TryFrom;

use just_core::math::Vec3;

struct Renderable {
    mesh: String,
}

struct Object {
    name: String,
    position: Option<Vec3>,
    renderable: Option<Renderable>,
    script: Option<String>,
    children: Option<Vec<Object>>,
    scale: Option<Vec3>,
}

impl Object {
    fn new(name: String) -> Object {
        Object {
            name,
            position: None,
            renderable: None,
            script: None,
            children: None,
            scale: None,
        }
    }
}

fn try_get_value<T: Copy>(
    scope: &mut v8::HandleScope,
    parent: &v8::Local<v8::Object>,
    name: &str,
) -> Result<Option<T>, String> {
    let key = v8::String::new(scope, name).unwrap();
    if let Some(pos_val) = parent
        .get(scope, key.into())
        .filter(|x| x.is_object())
        .and_then(|x| x.to_object(scope))
    {
        if pos_val.internal_field_count() != 1 {
            return Result::Err(format!(
                "'{}' argument must be of type {}",
                name,
                std::any::type_name::<T>()
            ));
        }
        let internal = pos_val.get_internal_field(scope, 0).unwrap();
        let external = v8::Local::<v8::External>::try_from(internal).unwrap();
        let data_ptr = external.value();
        let vec3_ptr: *mut T = data_ptr as _;
        return Ok(Some(unsafe { *vec3_ptr }));
    }
    Ok(None)
}

fn hacky_js_creator<'a>(
    scope: &mut v8::HandleScope<'a>,
    args: v8::FunctionCallbackArguments<'a>,
    mut rv: v8::ReturnValue,
) {
    match (|| -> Result<(), String> {
        if args.length() != 1 {
            return Result::Err("aniasFuntion accepts exactly one argument!".to_owned());
        }

        let arg = args.get(0);

        if !arg.is_object() {
            return Result::Err("aniasFuntion wants argument to be an object".to_owned());
        }
        let obj = arg.to_object(scope).unwrap();

        let name_key = v8::String::new(scope, "name").unwrap();
        let name = match obj.get(scope, name_key.into()) {
            Some(x) if !x.is_null_or_undefined() => x,
            _ => {
                return Result::Err("aniasFuntion requires object to have string property named 'name'.".to_owned());
            }
        };
        let mut obj_data = Object::new(name.to_string(scope).unwrap().to_rust_string_lossy(scope));

        if let Some(x) = try_get_value::<Vec3>(scope, &obj, "position")? {
            obj_data.position = Some(x);
        }

        if let Some(x) = try_get_value::<Vec3>(scope, &obj, "scale")? {
            obj_data.scale = Some(x);
        }

        let mesh_key = v8::String::new(scope, "mesh").unwrap();
        if let Some(x) = obj
            .get(scope, mesh_key.into())
            .filter(|x| x.is_string())
            .map(|x| x.to_string(scope).unwrap().to_rust_string_lossy(scope))
        {
            obj_data.renderable = Some(Renderable { mesh: x });
        }

        Result::Ok(())
    })() {
        Ok(_) => {}
        Err(msg) => {
            let msg = v8::String::new(scope, &msg).unwrap();
            let exception = v8::Exception::error(scope, msg);
            scope.throw_exception(exception);
        }
    }
}

pub fn create_hacky_creator(scope: &mut v8::HandleScope, context: v8::Global<v8::Context>) {
    let namespace_key = v8::String::new(scope, "World").unwrap();
    let context = context.open(scope);
    let world_namespace = context.global(scope).get(scope, namespace_key.into()).unwrap();

    let function = v8::Function::builder(hacky_js_creator).build(scope).unwrap();
    let function_key = v8::String::new(scope, "aniasFunction").unwrap();
    assert!(world_namespace.is_object());
    let world_namespce_obj = world_namespace.to_object(scope).unwrap();

    world_namespce_obj.set(scope, function_key.into(), function.into());
}
