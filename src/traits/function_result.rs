pub trait ResultEncoder {
    type ResultType;
    
    fn empty(&mut self) -> Self::ResultType;

    fn encode_float(&mut self, value: f32) -> Self::ResultType;

    fn encode_bool(&mut self, value: bool) -> Self::ResultType;

    fn encode_i32(&mut self, value: i32) -> Self::ResultType;

    fn encode_external_type<T>(&mut self, value: T) -> Self::ResultType;
}

pub trait FunctionResult: Sized{
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

impl FunctionResult for () {
    fn into_script_value<PE: ResultEncoder>(self, enc: &mut PE) -> PE::ResultType {
        enc.empty()
    }
}
