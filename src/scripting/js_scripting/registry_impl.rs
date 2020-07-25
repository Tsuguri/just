use crate::traits::{
    ScriptApiRegistry,
    FunctionParameter,
    FunctionResult,
    TypeCreationError,
};
use super::js::{
    Property,
    ContextGuard,
    value::{
        Value,
        Object,
        Number,
        null,
        Function,
        Boolean,
        function::CallbackInfo,
    },
};
use super::api_helpers;
use legion::prelude::*;
use super::{JsScriptEngine, JsRuntimeError};

struct JsResultEncoder<'a> {
    guard: &'a ContextGuard<'a>,
}

impl<'a> crate::traits::ResultEncoder for JsResultEncoder<'a> {
    type ResultType = Value;

    fn empty(&mut self) -> Self::ResultType {
        null(&self.guard)
    }

    fn encode_float(&mut self, value: f32) -> Self::ResultType {
        Number::from_double(&self.guard, value as f64).into()
    }

    fn encode_bool(&mut self, value: bool) -> Self::ResultType {
        Boolean::new(&self.guard, value).into()
    }

    fn encode_i32(&mut self, value: i32) -> Self::ResultType {
        Number::new(&self.guard, value).into()
    }
}

struct JsParamSource<'a> {
    guard: &'a ContextGuard<'a>,
    params: CallbackInfo,
    world: &'a World,
    current: usize,
}

impl<'a> crate::traits::ParametersSource for JsParamSource<'a> {
    type ErrorType = JsRuntimeError;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType> {
        let value = self.params.arguments[self.current].clone().into_number().ok_or(JsRuntimeError::WrongTypeParameter)?.value_double() as f32;
        self.current +=1;
        Result::Ok(value)
    }

    fn read_bool(&mut self) -> Result<bool, Self::ErrorType> {
        let val = self.params.arguments[self.current].clone().into_boolean().ok_or(JsRuntimeError::WrongTypeParameter)?.value();
        self.current +=1;
        Result::Ok(val)
    }

    fn read_i32(&mut self) -> Result<i32, Self::ErrorType> {
        let value = self.params.arguments[self.current].clone().into_number().ok_or(JsRuntimeError::WrongTypeParameter)?.value() as i32;
        self.current +=1;
        Result::Ok(value)
    }

    fn read_all<T: FunctionParameter>(&mut self) -> Result<Vec<T>, Self::ErrorType> {
        if self.current >= self.params.arguments.len() {
            return Result::Ok(vec![]);
        }
        let mut args = Vec::with_capacity(self.params.arguments.len() - self.current);
        while self.current < self.params.arguments.len() {
            args.push(T::read(self)?);
        }
        Result::Ok(args)
    }

    fn read_formatted(&mut self) -> Result<String, Self::ErrorType> {
        let value = self.params.arguments[self.current].to_string(self.guard);
        self.current +=1;
        Result::Ok(value)
    }

    fn read_system_data<T: 'static + Send + Sync>(&mut self) -> Result<legion::resource::FetchMut<T>, Self::ErrorType> {
        Result::Ok(self.world.resources.get_mut::<T>().unwrap())
    }


    /*
    fn read_component<'b, T: 'b + Send + Sync>(&'b mut self, id: Entity) -> Result<&'b T, JsRuntimeError> {
        let external = self.params.this.into_external().ok_or(JsRuntimeError::WrongTypeParameter)?;
        let this = unsafe { external.value::<game_object_api::GameObjectData>() };
        let ctx = self.guard.context();

        let world = api_helpers::world(&ctx);
        let component = world.get_component::<T>(this.id).ok_or(JsRuntimeError::WrongTypeParameter)?;
        Result::Ok(&component)
    }
    */
}

impl<'a> JsParamSource<'a> {
    pub fn create(guard: &'a ContextGuard<'a>, params: CallbackInfo, world: &'a World) -> Self {
        Self {
            guard,
            params,
            current: 0,
            world
        }
    }

}

impl ScriptApiRegistry for JsScriptEngine {
    type Namespace = Object;
    type Type = Value;

    //type ParamEncoder = Para
    type ErrorType = JsRuntimeError;

    fn register_namespace(&mut self, name: &str, parent: Option<&Self::Namespace>) -> Self::Namespace {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();
        let par = match parent {
            Some(x) => x,
            None => &global,
        };
        let ns = Object::new(&guard);
        par.set(&guard, Property::new(&guard, name), ns.clone());
        ns
    }


    fn register_function<P, R, F>(&mut self, name: &str, namespace: Option<&Self::Namespace>, fc: F)
    where P: crate::traits::FunctionParameter,
          R: crate::traits::FunctionResult,
          F: 'static + Send + Sync + Fn(P)-> R {
        let guard = self.context.make_current().unwrap();
        let fun = Function::new(&guard, Box::new(move |gd, params|{
            let ctx = gd.context();
            let world = api_helpers::world(&ctx);
            let mut param_source = JsParamSource::create(gd, params, world);
            let ret = fc(P::read(&mut param_source).unwrap()); // map to js exception?
            drop(param_source);
            let mut enc = JsResultEncoder{guard: gd};
            Result::Ok(ret.into_script_value(&mut enc))
        }));
        let global = guard.global();

        let parent = match namespace {
            Some(x) => x,
            None => &global,
        };
        parent.set(&guard, Property::new(&guard, name), fun);
        
    }
     fn register_native_type<T: 'static>(&mut self, name: &str, namespace: Option<&Self::Namespace>) -> Result<Self::Type,TypeCreationError> {
        let guard = self.context.make_current().unwrap();
        let type_id = std::any::TypeId::of::<T>();
        if self.external_types_prototypes.contains_key(&type_id) {
            return Err(TypeCreationError::TypeAlreadyRegistered);
        }

        let prototype = Object::new(&guard);
        self.external_types_prototypes.insert(type_id, prototype.clone());

        Result::Ok(prototype.into())
     }

     fn register_static_property<P1, P2, R, F1, F2>(
         &mut self,
         name: &str,
         namespace: Option<&Self::Namespace>,
         getter: Option<F1>,
         setter: Option<F2>)
     where P1: FunctionParameter,
           P2: FunctionParameter,
           R: FunctionResult,
           F1: 'static + Send + Sync + Fn(P1)->R,
           F2: 'static + Send + Sync + Fn(P2) {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();

        let property_object = Object::new(&guard);

        match getter {
            None => (),
            Some(func) => {
                let fun = Function::new(&guard, Box::new(move |gd, params|{
                    let ctx = gd.context();
                    let world = api_helpers::world(&ctx);
                    let mut param_source = JsParamSource::create(gd, params, world);
                    let ret = func(P1::read(&mut param_source).unwrap());
                    let mut enc = JsResultEncoder{guard: gd};
                    Result::Ok(ret.into_script_value(&mut enc))
                }));
                property_object.set(&guard, Property::new(&guard, "get"), fun);
            }
        }
        match setter {
            None => (),
            Some(func) => {
                let fun = Function::new(&guard, Box::new(move |gd, params|{
                    let ctx = gd.context();
                    let world = api_helpers::world(&ctx);
                    let mut param_source = JsParamSource::create(gd, params, world);
                    func(P2::read(&mut param_source).unwrap()); // map to js exception?
                    Result::Ok(null(gd))
                }));
                property_object.set(&guard, Property::new(&guard, "set"), fun);
            }
        }
        let par = match namespace {
            Some(x) => x,
            None => &global,
        };

        par.define_property(&guard, Property::new(&guard, name), property_object);
     }
}
