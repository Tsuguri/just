use std::{convert::TryInto, path::Path};

pub struct ScriptFactory {}

impl ScriptFactory {
    pub fn from_code<'a>(
        scope: &mut v8::HandleScope<'a>,
        name: String,
        _path: &Path,
        code: &str,
    ) -> Result<v8::Local<'a, v8::Function>, ()> {
        // let def = js::script::parse(guard, code)?;
        // let factory = def.construct(&guard, guard.global(), &[])?;
        // let factory = match factory.into_function() {
        //     Some(elem) => elem,
        //     None => return Result::Err(js::Error::ScriptCompilation("Not a function".to_string())),
        // };

        let try_catch = &mut v8::TryCatch::new(scope);
        let code = v8::String::new(try_catch, code).unwrap();
        let result = v8::Script::compile(try_catch, code, None)
            .and_then(|script| script.run(try_catch))
            .map_or_else(|| Err(try_catch.stack_trace().unwrap()), Ok);

        println!("Looking for class {}", name);

        match result {
            Ok(v) => {
                let context = try_catch.get_current_context();
                let global = context.global(try_catch);
                let expected_name = v8::String::new(try_catch, &name).unwrap();
                println!(
                    "context has: {}, {:?}",
                    name,
                    global.has(try_catch, expected_name.into())
                );
                let value = global.get(try_catch, expected_name.into());
                if !value.is_some() {
                    //no expected name
                    return Err(());
                }
                let value = value.unwrap();
                println!("Odpaliło się");
                println!("undefined: {:#?}", value.is_undefined());
                println!("null: {:#?}", value.is_null());
                println!("function: {}", value.is_function());
                if value.is_function() {
                    Ok(v.try_into().unwrap())
                } else {
                    println!("is not a function");
                    Err(())
                }
            }
            Err(_) => Err(()),
        }
        // Result::Ok(factory)
    }
    pub fn from_path<'a>(scope: &mut v8::HandleScope<'a>, path: &Path) -> Result<v8::Local<'a, v8::Function>, ()> {
        println!("loading code: {}", path.display());
        let name = path.file_stem().unwrap().to_str().unwrap().to_owned();

        let code = std::fs::read_to_string(path).unwrap();
        Self::from_code(scope, name, path, &code)
    }
}
