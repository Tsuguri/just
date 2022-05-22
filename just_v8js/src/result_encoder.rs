use std::ffi::c_void;

use just_core::traits::scripting::ResultEncoder;

use crate::EHM;

pub struct V8ResultEncoder<'a, 'b> {
    scope: &'a mut v8::HandleScope<'b>,
}

impl<'a, 'b> V8ResultEncoder<'a, 'b> {
    pub fn new(scope: &'a mut v8::HandleScope<'b>) -> Self {
        V8ResultEncoder { scope }
    }
}

impl<'a, 'b> ResultEncoder for V8ResultEncoder<'a, 'b> {
    type ResultType = v8::Local<'b, v8::Value>;

    fn empty(&mut self) -> Self::ResultType {
        v8::null(self.scope).into()
    }

    fn encode_float(&mut self, value: f32) -> Self::ResultType {
        v8::Number::new(self.scope, value as f64).into()
    }

    fn encode_bool(&mut self, value: bool) -> Self::ResultType {
        v8::Boolean::new(self.scope, value).into()
    }

    fn encode_i32(&mut self, value: i32) -> Self::ResultType {
        v8::Number::new(self.scope, value as f64).into()
    }

    fn encode_external_type<T: 'static>(&mut self, value: T) -> Self::ResultType {
        let protos = self.scope.get_slot::<&EHM>().unwrap();
        let prototypes = *protos;
        let obj = prototypes.get_prototype::<T>().open(self.scope).new_instance(self.scope).unwrap();
        let ext = v8::External::new(self.scope, Box::into_raw(Box::new(value)) as *mut c_void);
        obj.set_internal_field(0, ext.into());

        obj.into()
    }

    fn encode_string(&mut self, value: &str) -> Self::ResultType {
        v8::String::new(self.scope, value).unwrap().into()
    }

    fn encode_array(&mut self, value: Vec<Self::ResultType>) -> Self::ResultType {
        let res = v8::Array::new(self.scope, value.len() as i32);
        for (id, val) in value.into_iter().enumerate() {
            res.set_index(self.scope, id as u32, val);
        }
        res.into()
    }
}
