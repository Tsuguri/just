pub trait ResultEncoder {
    type ResultType;

    fn empty(&mut self) -> Self::ResultType;

    fn encode_float(&mut self, value: f32) -> Self::ResultType;

    fn encode_bool(&mut self, value: bool) -> Self::ResultType;

    fn encode_i32(&mut self, value: i32) -> Self::ResultType;

    fn encode_external_type<T: 'static>(&mut self, value: T) -> Self::ResultType;

    fn encode_string(&mut self, value: &str) -> Self::ResultType;

    fn encode_array(&mut self, value: Vec<Self::ResultType>) -> Self::ResultType;
}

pub trait FunctionResult: Sized + 'static {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.encode_external_type(self)
    }
}

impl FunctionResult for f32 {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.encode_float(self)
    }
}

impl FunctionResult for i32 {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.encode_i32(self)
    }
}

impl FunctionResult for usize {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.encode_i32(self as i32)
    }
}

impl FunctionResult for bool {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.encode_bool(self)
    }
}

impl FunctionResult for String {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.encode_string(&self)
    }
}

impl<T: FunctionResult> FunctionResult for Vec<T> {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        let objs = self.into_iter().map(|x| x.into_script_value(enc)).collect();
        enc.encode_array(objs)
    }
}

impl<T: FunctionResult> FunctionResult for Option<T> {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        match self {
            Some(x) => x.into_script_value(enc),
            None => enc.empty(),
        }
    }
}

impl FunctionResult for () {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.empty()
    }
}

impl<T: nalgebra_glm::Scalar + Send + Sync> FunctionResult for nalgebra_glm::TVec2<T> {}
impl<T: nalgebra_glm::Scalar + Send + Sync> FunctionResult for nalgebra_glm::TVec3<T> {}
impl<T: nalgebra_glm::Scalar + Send + Sync> FunctionResult for nalgebra_glm::TVec4<T> {}
