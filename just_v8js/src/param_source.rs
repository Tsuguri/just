use just_core::{ecs::prelude::World, traits::scripting::ParametersSource};

pub struct V8ParametersSource<'a, 'b, 'c> {
    scope: &'a mut v8::HandleScope<'b>,
    arguments: &'a v8::FunctionCallbackArguments<'c>,
    current: usize,
}

impl<'a, 'b, 'c> V8ParametersSource<'a, 'b, 'c> {
    pub fn new(scope: &'a mut v8::HandleScope<'b>, arguments: &'a v8::FunctionCallbackArguments<'c>) -> Self {
        Self {
            scope,
            arguments,
            current: 0,
        }
    }
}

impl<'a, 'b, 'c> ParametersSource for V8ParametersSource<'a, 'b, 'c> {
    type ErrorType = i32;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType> {
        if self.arguments.length() <= self.current as i32 {
            return Result::Err(0);
        }
        let value = self.arguments.get(self.current as i32);
        if !value.is_number() {
            return Result::Err(1);
        }
        self.current += 1;
        return Ok(value.to_number(self.scope).unwrap().value() as f32);
    }

    fn read_bool(&mut self) -> Result<bool, Self::ErrorType> {
        if self.arguments.length() <= self.current as i32 {
            return Result::Err(0);
        }
        let value = self.arguments.get(self.current as i32);
        if !value.is_boolean() {
            return Result::Err(1);
        }
        self.current += 1;
        return Ok(value.to_boolean(self.scope).boolean_value(self.scope));
    }

    fn read_i32(&mut self) -> Result<i32, Self::ErrorType> {
        if self.arguments.length() <= self.current as i32 {
            return Result::Err(0);
        }
        let value = self.arguments.get(self.current as i32);
        if !value.is_number() {
            return Result::Err(1);
        }
        self.current += 1;
        match value.to_number(self.scope).unwrap().int32_value(self.scope) {
            None => Result::Err(2),
            Some(x) => Result::Ok(x),
        }
    }

    fn read_formatted(&mut self) -> Result<String, Self::ErrorType> {
        if self.arguments.length() <= self.current as i32 {
            return Result::Err(0);
        }
        let string = self.arguments.get(self.current as i32).to_rust_string_lossy(self.scope);
        self.current += 1;
        Result::Ok(string)
    }

    fn read_all<T: just_core::traits::scripting::FunctionParameter>(&mut self) -> Result<Vec<T>, Self::ErrorType> {
        if self.current as i32 >= self.arguments.length() {
            return Result::Ok(vec![]);
        }
        let mut args = Vec::with_capacity(self.arguments.length() as usize - self.current);
        while self.current < self.arguments.length() as usize {
            args.push(T::read(self)?);
        }
        Result::Ok(args)
    }

    fn read_system_data<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<just_core::ecs::resource::FetchMut<T>, Self::ErrorType> {
        Result::Ok(
            self.scope
                .get_slot_mut::<&mut World>()
                .unwrap()
                .resources
                .get_mut::<T>()
                .unwrap(),
        )
    }

    fn read_world(&mut self) -> Result<&mut just_core::ecs::world::World, Self::ErrorType> {
        //Result::Ok(&mut self.world)
        Result::Ok(self.scope.get_slot_mut::<&mut World>().unwrap())
    }

    fn read_native<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType> {
        todo!()
    }

    fn read_native_this<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType> {
        todo!()
    }

    fn read_component<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<just_core::ecs::borrow::RefMut<T>, Self::ErrorType> {
        todo!()
    }

    fn read_component_this<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<just_core::ecs::borrow::RefMut<T>, Self::ErrorType> {
        todo!()
    }

    fn is_null(&self) -> bool {
        self.arguments.length() <= self.current as i32 || self.arguments.get(self.current as i32).is_null()
    }
}
