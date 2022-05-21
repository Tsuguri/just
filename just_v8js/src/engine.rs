use crate::game_object_api::GameObjectApi;
use crate::V8ApiRegistry;
use just_core::ecs::prelude::*;
use std::collections::HashMap;
use std::convert::TryInto;

use crate::JsScript;
use crate::{script_creation::ScriptCreationQueue, script_factory::ScriptFactory};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct JsEngineConfig {
    pub source_root: String,
    pub v8_args: Vec<String>,
}

pub struct V8Engine {
    isolate: v8::OwnedIsolate,
    context: v8::Global<v8::Context>,
    //prototypes: EHM,
    controllers_constructors: HashMap<String, v8::Global<v8::Function>>,
}

fn println_callback(_scope: &mut v8::HandleScope, _args: v8::FunctionCallbackArguments, _rv: v8::ReturnValue) {
    println!("hard stuff");
}

impl V8Engine {
    fn create_global_object_template<'a>(
        higher_scope: &mut v8::HandleScope<'a, ()>,
    ) -> v8::Local<'a, v8::ObjectTemplate> {
        let scope = &mut v8::EscapableHandleScope::new(higher_scope);
        let global_template = v8::ObjectTemplate::new(scope);

        // create built in methods?
        let key = v8::String::new(scope, "print").unwrap();
        let val = v8::FunctionTemplate::new(scope, println_callback);
        // val.set_name(key);
        global_template.set(key.into(), val.into());

        scope.escape(global_template)
    }

    pub fn create_without_api(args: JsEngineConfig) -> Self {
        Self::create_with_api(args, |_| {})
    }

    pub fn create_with_api<Builder>(args: JsEngineConfig, builder: Builder) -> Self
    where
        Builder: for<'a, 'b, 'c> FnOnce(&'c mut V8ApiRegistry<'a, 'b>),
    {
        if !args.v8_args.is_empty() {
            v8::V8::set_flags_from_command_line(args.v8_args);
        }
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();

        let create_params = v8::Isolate::create_params();
        let mut isolate = v8::Isolate::new(create_params);
        let context = {
            let mut scope = v8::HandleScope::new(&mut isolate);
            let global_template = Self::create_global_object_template(&mut scope);
            //fill_api(&mut registry);
            let context = v8::Context::new_from_template(&mut scope, global_template);
            v8::Global::new(&mut scope, context)
        };
        let mut controllers_constructors = HashMap::new();
        {
            let scope = &mut v8::HandleScope::with_context(&mut isolate, context.clone());
            Self::load_at_path(
                &mut controllers_constructors,
                scope,
                &"user",
                std::path::Path::new(&args.source_root),
            )
            .unwrap();
        }
        {
            let mut scope = v8::HandleScope::with_context(&mut isolate, context.clone());
            let mut sar = V8ApiRegistry {
                scope: &mut scope,
                context: context.clone(),
                things: Default::default(),
                object_templates: Default::default(),
                last_id: 0,
            };
            GameObjectApi::register(&mut sar);
            builder(&mut sar);
        }

        for item in controllers_constructors.iter() {
            println!("   loaded controller: {}", item.0);
        }

        Self {
            isolate,
            context,
            // prototypes: Default::default(),
            controllers_constructors,
        }
    }

    pub fn run(&mut self, source: &str) {
        let scope = &mut v8::HandleScope::with_context(&mut self.isolate, self.context.clone());

        let try_catch = &mut v8::TryCatch::new(scope);
        let code = v8::String::new(try_catch, source).unwrap();
        let result = v8::Script::compile(try_catch, code, None)
            .and_then(|script| script.run(try_catch))
            .map_or_else(|| Err(try_catch.stack_trace().unwrap()), Ok);
        let strong = match result {
            Ok(v) => Ok(v.to_string(try_catch).unwrap().to_rust_string_lossy(try_catch)),
            Err(e) => Err(e.to_string(try_catch).unwrap().to_rust_string_lossy(try_catch)),
        };
        println!("{:?}", strong);
    }

    fn load_at_path(
        constructors: &mut HashMap<String, v8::Global<v8::Function>>,
        scope: &mut v8::HandleScope,
        namespace: &str,
        directory: &std::path::Path,
    ) -> Result<(), &'static str> {
        println!("loading scripts from: {:?} into {}", directory, namespace);

        let paths = std::fs::read_dir(directory).map_err(|_| "counldn't read directory")?;
        for path in paths {
            let path = path.map_err(|_| "error reading script directory")?;

            if path.path().is_dir() {
                let p = path.path();
                let p2 = p.file_stem().unwrap();
                let namespace_suffix = match p2.to_str() {
                    Option::None => return Result::Err("invalid character in namespace string"),
                    Option::Some(name) => name,
                };
                println!("creating namespace: {:?}", namespace_suffix);
                Self::load_at_path(
                    constructors,
                    scope,
                    &format!("{}.{}", namespace, namespace_suffix),
                    &path.path(),
                )?;
            } else {
                let p = path.path();
                let p2 = p.file_stem().unwrap().to_str().unwrap();
                let controller_name = format!("{}.{}", namespace, p2);
                println!("loading controller: {}", controller_name);
                let obj = ScriptFactory::from_path(scope, &p).unwrap();
                constructors.insert(controller_name, v8::Global::new(scope, obj));
            }
        }

        Result::Ok(())
    }

    pub fn create<Builder>(config: JsEngineConfig, world: &mut World, builder: Builder) -> Self
    where
        Builder: for<'a, 'b, 'c> FnOnce(&'c mut V8ApiRegistry<'a, 'b>),
    {
        world.resources.insert(ScriptCreationQueue { q: vec![] });
        V8Engine::create_with_api(config, |sar| builder(sar))
    }

    pub fn create_script(&mut self, gameobject_id: Entity, typ: &str, world: &mut World) {
        let script_name = format!("user.{}", typ);
        println!("Attempting to create controllr: {}", script_name);

        if self.controllers_constructors.contains_key(&script_name) {
            println!("  Found requested controller");
            let scope = &mut v8::HandleScope::with_context(&mut self.isolate, self.context.clone());
            let constructor = self.controllers_constructors.get(&script_name).unwrap();
            //let recv = v8::null(scope);
            let controller = constructor.open(scope).new_instance(scope, &vec![]);

            match controller {
                Some(x) => {
                    if x.is_object() {
                        println!("It's an object!!");
                    } else {
                        println!("  Ugh");
                    }

                    let glob_constructor = v8::Global::new(scope, x);
                    let comp = JsScript {
                        object: glob_constructor,
                    };
                    world.add_component(gameobject_id, comp);
                }
                None => {
                    println!("Failed to create {}", script_name);
                }
            }
        } else {
            println!("  Missing controller: {}", script_name);
        }
    }

    pub fn update(&mut self, world: &mut World) {
        // update scripts

        let mut scope = v8::HandleScope::with_context(&mut self.isolate, self.context.clone());
        let reference = unsafe { std::mem::transmute::<&mut World, &'static mut World>(world) };
        scope.set_slot::<&'static mut World>(reference);

        let query = <Read<JsScript>>::query();
        for script in query.iter_immutable(world) {
            let controller = script.object.open(&mut scope);
            let handl = script.object.clone();
            let key = v8::String::new(&mut scope, "update").unwrap();

            let update = controller.get(&mut scope, key.into());
            match update {
                Some(x) => {
                    if x.is_function() {
                        let try_catch = &mut v8::TryCatch::new(&mut scope);
                        let func: v8::Local<v8::Function> = x.try_into().unwrap();
                        let script_handle = v8::Local::new(try_catch, handl);
                        func.call(try_catch, script_handle.into(), &vec![]);
                    } else {
                        println!("update is not function");
                    }
                }
                None => {
                    println!("no update here");
                }
            }
        }
        scope.remove_slot::<&'static mut World>();

        drop(query);
        drop(scope);

        // creating scripts requested in this frame
        let to_create: Vec<_> = std::mem::take(&mut world.resources.get_mut::<ScriptCreationQueue>().unwrap().q);

        for data in to_create {
            self.create_script(data.object, &data.script_type, world);
        }
    }
}
