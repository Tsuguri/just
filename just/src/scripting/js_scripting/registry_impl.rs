use super::api_helpers;
use super::js::{
    value::{
        function::CallbackInfo, null, Boolean, External, Function, Number, Object,
        String as JsString, Value,
    },
    ContextGuard, Property,
};
use super::EHM;
use super::{JsRuntimeError, JsScriptEngine};

use just_traits::scripting::{
    ScriptApiRegistry,
    FunctionResult,
    FunctionParameter,
    TypeCreationError,
    ResultEncoder,
    ParametersSource,
};
use just_core::ecs::prelude::*;
use just_core::ecs;

struct JsResultEncoder<'a> {
    guard: &'a ContextGuard<'a>,
    external_prototypes: &'a EHM,
}

impl<'a> ResultEncoder for JsResultEncoder<'a> {
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

    fn encode_external_type<T: 'static>(&mut self, value: T) -> Self::ResultType {
        let obj = External::new(&self.guard, Box::new(value));
        obj.set_prototype(&self.guard, self.external_prototypes[&std::any::TypeId::of::<T>()].clone()).unwrap();
        obj.into()
    }

    fn encode_string(&mut self, value: &str) -> Self::ResultType {
        JsString::new(&self.guard, &value).into()
    }
}

struct JsParamSource<'a> {
    guard: &'a ContextGuard<'a>,
    params: CallbackInfo,
    world: &'a mut World,
    current: usize,
}

impl<'a> ParametersSource for JsParamSource<'a> {
    type ErrorType = JsRuntimeError;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType> {
        let value = self.params.arguments[self.current]
            .clone()
            .into_number()
            .ok_or(JsRuntimeError::WrongTypeParameter)?
            .value_double() as f32;
        self.current += 1;
        Result::Ok(value)
    }

    fn read_bool(&mut self) -> Result<bool, Self::ErrorType> {
        let val = self.params.arguments[self.current]
            .clone()
            .into_boolean()
            .ok_or(JsRuntimeError::WrongTypeParameter)?
            .value();
        self.current += 1;
        Result::Ok(val)
    }

    fn read_i32(&mut self) -> Result<i32, Self::ErrorType> {
        let value = self.params.arguments[self.current]
            .clone()
            .into_number()
            .ok_or(JsRuntimeError::WrongTypeParameter)?
            .value() as i32;
        self.current += 1;
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
        self.current += 1;
        Result::Ok(value)
    }

    fn read_system_data<T: 'static + Send + Sync>(
        &mut self,
    ) -> Result<ecs::resource::FetchMut<T>, Self::ErrorType> {
        Result::Ok(self.world.resources.get_mut::<T>().unwrap())
    }

    fn read_world(&mut self) -> Result<&mut World, Self::ErrorType> {
        Result::Ok(&mut self.world)
    }

    fn read_native<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType> {
        let native = self.params.arguments[self.current]
            .clone()
            .into_external()
            .ok_or(JsRuntimeError::WrongTypeParameter)?;
        self.current += 1;
        Result::Ok(unsafe { std::mem::transmute::<&mut T, &'static mut T>(native.value::<T>()) })
    }

    fn read_native_this<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<&mut T, Self::ErrorType> {
        let native = self
            .params
            .this
            .clone()
            .into_external()
            .ok_or(JsRuntimeError::WrongTypeParameter)?;
        Result::Ok(unsafe { std::mem::transmute::<&mut T, &'static mut T>(native.value::<T>()) })
    }

    fn read_component<T: 'static + Send + Sync + Sized>(&mut self) -> Result<ecs::borrow::RefMut<T>, Self::ErrorType> {
        let native = self.params.arguments[self.current]
            .clone()
            .into_external()
            .ok_or(JsRuntimeError::WrongTypeParameter)?;
        self.current += 1;
        let component_info = unsafe {native.value::<ComponentHandle>()};
        let go_id = component_info.id;

        match self.world.get_component_mut(go_id) {
            None => {
                Result::Err(JsRuntimeError::ComponentNotPresent)
            }
            Some(x) => {
                Result::Ok(x)
            }
        }
        
    }

    fn read_component_this<T: 'static + Send + Sync + Sized>(
            &mut self,
        ) -> Result<ecs::borrow::RefMut<T>, Self::ErrorType> {
        let native = self
            .params
            .this
            .clone()
            .into_external()
            .ok_or(JsRuntimeError::WrongTypeParameter)?;
        let component_info = unsafe {native.value::<ComponentHandle>()};
        let go_id = component_info.id;

        match self.world.get_component_mut(go_id) {
            None => {
                Result::Err(JsRuntimeError::ComponentNotPresent)
            }
            Some(x) => {
                Result::Ok(x)
            }
        }
    }

    fn is_null(&self) -> bool {
        self.params.arguments.len() <= self.current || self.params.arguments[self.current].is_null()
    }
}

impl<'a> JsParamSource<'a> {
    pub fn create(guard: &'a ContextGuard<'a>, params: CallbackInfo, world: &'a mut World) -> Self {
        Self {
            guard,
            params,
            current: 0,
            world,
        }
    }
}

struct ComponentCreationData {
    magic_value: f64,
    create: Box<dyn Fn(&mut World, Entity)>,
    delete: Box<dyn Fn(&mut World, Entity)>,
    get: Box<dyn Fn(&mut World, Entity) -> Option<ComponentHandle>>,
    handle_prototype: Object,
}

struct ComponentHandle {
    id: Entity,
}

impl JsScriptEngine {
    pub fn create_component_api(&mut self) {
        let guard = self.context.make_current().unwrap();
        let game_object_id = std::any::TypeId::of::<super::game_object_api::GameObjectData>();
        let game_object_prototype = self.external_types_prototypes[&game_object_id].clone();

        assert!(!game_object_prototype.is_null());
        let fun = Function::new(
            &guard,
            Box::new(move |gd, params| {
                let ctx = gd.context();
                let mut world = api_helpers::world(&ctx);
                let go = params.this.clone().into_external().unwrap();
                let go_data = unsafe {go.value::<super::game_object_api::GameObjectData>()};
                let go_id = go_data.id;

                if params.arguments.len() == 0 {
                    return Result::Err(null(gd));
                }

                let component_this = params.arguments[0].clone().into_external();
                match component_this {
                    Some(x)=>{
                        unsafe {
                            let data = x.value::<ComponentCreationData>();
                            if data.magic_value != 12312.1f64 {
                                return Result::Err(null(gd));
                            }
                            match (data.get)(&mut world,go_id) {
                                None => return Result::Ok(null(gd)),
                                Some(y) => {
                                    let handle = External::new(gd, Box::new(y));
                                    handle.set_prototype(gd, data.handle_prototype.clone()).unwrap();
                                    return Result::Ok(handle.into());
                                }
                            }
                        }
                    },
                    None =>{
                        return Result::Err(null(gd));
                        // passed argument is not component creation data.
                        // for now it may be script class.
                    },
                };
            }),
        );
        game_object_prototype.set(&guard, Property::new(&guard, "getComponent"), fun);

        let fun = Function::new(
            &guard,
            Box::new(move |gd, params| {
                let ctx = gd.context();
                let mut world = api_helpers::world(&ctx);
                let go = params.this.clone().into_external().unwrap();
                let go_data = unsafe{go.value::<super::game_object_api::GameObjectData>()};
                let go_id = go_data.id;

                if params.arguments.len() == 0 {
                    return Result::Err(null(gd));
                }

                let component_this = params.arguments[0].clone().into_external();
                match component_this {
                    Some(x)=>{
                        unsafe {
                            let data = x.value::<ComponentCreationData>();
                            if data.magic_value != 12312.1f64 {
                                return Result::Err(null(gd));
                            }
                            (data.delete)(&mut world, go_id);
                        }
                    },
                    None => (),
                };
                Result::Ok(null(gd))
            }),
        );
        game_object_prototype.set(&guard, Property::new(&guard, "deleteComponent"), fun);

        let fun = Function::new(
            &guard,
            Box::new(move |gd, params| {
                let ctx = gd.context();
                let mut world = api_helpers::world(&ctx);
                let go = params.this.clone().into_external().unwrap();
                let go_data = unsafe {go.value::<super::game_object_api::GameObjectData>()};
                let go_id = go_data.id;

                if params.arguments.len() == 0 {
                    return Result::Err(null(gd));
                }

                let component_this = params.arguments[0].clone().into_external();
                match component_this {
                    Some(x)=>{
                        unsafe {
                            let data = x.value::<ComponentCreationData>();
                            if data.magic_value != 12312.1f64 {
                                println!("magic value not present");
                                println!("magic value: {}", data.magic_value);
                                return Result::Err(null(gd));
                            }
                            (data.create)(&mut world, go_id);
                            return Result::Ok(null(gd));
                        }
                    },
                    None => {
                        println!("This is not external of GameObjectData");
                        return Result::Err(null(gd));
                    },
                };
            }),
        );
        game_object_prototype.set(&guard, Property::new(&guard, "createComponent"), fun);

    }
}


impl ScriptApiRegistry for JsScriptEngine {
    type Namespace = Object;
    type Type = Value;
    type NativeType = Object;

    //type ParamEncoder = Para
    type ErrorType = JsRuntimeError;

    fn register_namespace(
        &mut self,
        name: &str,
        parent: Option<&Self::Namespace>,
    ) -> Self::Namespace {
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
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R,
    {
        let guard = self.context.make_current().unwrap();
        let fun = Function::new(
            &guard,
            Box::new(move |gd, params| {
                let ctx = gd.context();
                let world = api_helpers::world(&ctx);
                let prototypes = api_helpers::external_prototypes(&ctx);
                let mut param_source = JsParamSource::create(gd, params, world);
                let ret = fc(P::read(&mut param_source).unwrap()); // map to js exception?
                drop(param_source);
                let mut enc = JsResultEncoder {
                    guard: gd,
                    external_prototypes: prototypes,
                };
                Result::Ok(ret.into_script_value(&mut enc))
            }),
        );
        let global = guard.global();

        let parent = match namespace {
            Some(x) => x,
            None => &global,
        };
        parent.set(&guard, Property::new(&guard, name), fun);
    }

    fn register_native_type<T, P, F>(
        &mut self,
        name: &str,
        namespace: Option<&Self::Namespace>,
        constructor: F,
    ) -> Result<Self::NativeType, TypeCreationError>
    where
        T: 'static,
        P: FunctionParameter,
        F: 'static + Send + Sync + Fn(P) -> T,
    {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();
        let type_id = std::any::TypeId::of::<T>();
        if self.external_types_prototypes.contains_key(&type_id) {
            return Err(TypeCreationError::TypeAlreadyRegistered);
        }
        let prototype = Object::new(&guard);
        let ret = prototype.clone();
        self.external_types_prototypes
            .insert(type_id, prototype.clone());
        let factory_function = Function::new(
            &guard,
            Box::new(move |g, args| {
                let ctx = g.context();
                let world = api_helpers::world(&ctx);
                let mut param_source = JsParamSource::create(g, args, world);
                let obj = External::new(
                    g,
                    Box::new(constructor(P::read(&mut param_source).unwrap())),
                );
                obj.set_prototype(g, prototype.clone()).unwrap();

                Result::Ok(obj.into())
            }),
        );
        let par = match namespace {
            Some(x) => x,
            None => &global,
        };
        par.set(&guard, Property::new(&guard, name), factory_function);

        Result::Ok(ret.into())
    }

    fn register_component<T, F>(
        &mut self,
        name: &str,
        namespace: Option<&Self::Namespace>,
        constructor: F,
    ) -> Result<Self::NativeType, TypeCreationError>
    where
        T: 'static + Send + Sync,
        F: 'static + Send + Sync + Fn() -> T,
    {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();
        let type_id = std::any::TypeId::of::<T>();
        if self.external_types_prototypes.contains_key(&type_id) {
            return Err(TypeCreationError::TypeAlreadyRegistered);
        }
        let prototype = Object::new(&guard);
        let ret = prototype.clone();
        self.external_types_prototypes
            .insert(type_id, prototype.clone());

        let creation_data = External::new(
            &guard,
            Box::new(ComponentCreationData {
                create: Box::new(move |world: &mut World, id: Entity| {
                    let comp = constructor();
                    world.add_component(id, comp);
                }),
                delete: Box::new(move |world: &mut World, id: Entity| {
                    world.remove_component::<T>(id);
                }),
                get: Box::new(|world: &mut World, id: Entity| {
                    let cp = world.get_component::<T>(id);
                    match cp {
                        None => return None,
                        Some(_) => return Some(ComponentHandle { id }),
                    }
                }),
                handle_prototype: prototype,
                magic_value: 12312.1f64 ,
            }),
        );

        let par = match namespace {
            Some(x) => x,
            None => &global,
        };
        par.set(&guard, Property::new(&guard, name), creation_data);

        Result::Ok(ret.into())
    }

    fn register_native_type_method<P, R, F>(
        &mut self,
        type_id: &Self::NativeType,
        name: &str,
        method: F,
    ) -> Result<(), TypeCreationError>
    where
        P: FunctionParameter,
        R: FunctionResult,
        F: 'static + Send + Sync + Fn(P) -> R,
    {
        let guard = self.context.make_current().unwrap();

        let fun = Function::new(
            &guard,
            Box::new(move |gd, params| {
                let ctx = gd.context();
                let world = api_helpers::world(&ctx);
                let prototypes = api_helpers::external_prototypes(&ctx);
                let mut param_source = JsParamSource::create(gd, params, world);
                let ret = method(P::read(&mut param_source).unwrap()); // map to js exception?
                drop(param_source);
                let mut enc = JsResultEncoder {
                    guard: gd,
                    external_prototypes: prototypes,
                };
                Result::Ok(ret.into_script_value(&mut enc))
            }),
        );
        type_id.set(&guard, Property::new(&guard, name), fun);

        Result::Ok(())
    }

    fn register_native_type_property<P1, P2, R, F1, F2>(
        &mut self,
        type_id: &Object,
        name: &str,
        getter: Option<F1>,
        setter: Option<F2>,
    ) where
        P1: FunctionParameter,
        P2: FunctionParameter,
        R: FunctionResult,
        F1: 'static + Send + Sync + Fn(P1) -> R,
        F2: 'static + Send + Sync + Fn(P2),
    {
        let guard = self.context.make_current().unwrap();

        let property_object = Object::new(&guard);

        match getter {
            None => (),
            Some(func) => {
                let fun = Function::new(
                    &guard,
                    Box::new(move |gd, params| {
                        let ctx = gd.context();
                        let world = api_helpers::world(&ctx);
                        let prototypes = api_helpers::external_prototypes(&ctx);
                        let mut param_source = JsParamSource::create(gd, params, world);
                        let ret = func(P1::read(&mut param_source).unwrap());
                        let mut enc = JsResultEncoder {
                            guard: gd,
                            external_prototypes: prototypes,
                        };
                        Result::Ok(ret.into_script_value(&mut enc))
                    }),
                );
                property_object.set(&guard, Property::new(&guard, "get"), fun);
            }
        }
        match setter {
            None => (),
            Some(func) => {
                let fun = Function::new(
                    &guard,
                    Box::new(move |gd, params| {
                        let ctx = gd.context();
                        let world = api_helpers::world(&ctx);
                        let mut param_source = JsParamSource::create(gd, params, world);
                        func(P2::read(&mut param_source).unwrap()); // map to js exception?
                        Result::Ok(null(gd))
                    }),
                );
                property_object.set(&guard, Property::new(&guard, "set"), fun);
            }
        }
        type_id.define_property(&guard, Property::new(&guard, name), property_object);
    }

    fn register_static_property<P1, P2, R, F1, F2>(
        &mut self,
        name: &str,
        namespace: Option<&Self::Namespace>,
        getter: Option<F1>,
        setter: Option<F2>,
    ) where
        P1: FunctionParameter,
        P2: FunctionParameter,
        R: FunctionResult,
        F1: 'static + Send + Sync + Fn(P1) -> R,
        F2: 'static + Send + Sync + Fn(P2),
    {
        let guard = self.context.make_current().unwrap();
        let global = guard.global();

        let property_object = Object::new(&guard);

        match getter {
            None => (),
            Some(func) => {
                let fun = Function::new(
                    &guard,
                    Box::new(move |gd, params| {
                        let ctx = gd.context();
                        let world = api_helpers::world(&ctx);
                        let prototypes = api_helpers::external_prototypes(&ctx);
                        let mut param_source = JsParamSource::create(gd, params, world);
                        let ret = func(P1::read(&mut param_source).unwrap());
                        let mut enc = JsResultEncoder {
                            guard: gd,
                            external_prototypes: prototypes,
                        };
                        Result::Ok(ret.into_script_value(&mut enc))
                    }),
                );
                property_object.set(&guard, Property::new(&guard, "get"), fun);
            }
        }
        match setter {
            None => (),
            Some(func) => {
                let fun = Function::new(
                    &guard,
                    Box::new(move |gd, params| {
                        let ctx = gd.context();
                        let world = api_helpers::world(&ctx);
                        let mut param_source = JsParamSource::create(gd, params, world);
                        func(P2::read(&mut param_source).unwrap()); // map to js exception?
                        Result::Ok(null(gd))
                    }),
                );
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
