use legion::prelude::*;

pub trait ParametersSource {
    type ErrorType : 'static + Send + Sync;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType>;

    fn read_bool(&mut self) -> Result<bool, Self::ErrorType>;

    fn read_i32(&mut self) -> Result<i32, Self::ErrorType>;

    fn read_formatted(&mut self) -> Result<String, Self::ErrorType>;

    fn read_all<T: FunctionParameter>(&mut self) -> Result<Vec<T>, Self::ErrorType>;

    fn read_system_data<T: 'static + Send + Sync + Sized>(&mut self) -> Result<legion::resource::FetchMut<T>, Self::ErrorType>;
    
    fn read_native_this<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType>;

    fn read_native<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType>;
}

pub trait FunctionParameter: Sized{
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType>;
}

pub struct Data<'a, T: 'static + Send + Sync> {
    pub fetch: legion::resource::FetchMut<'a, T>,
}

pub struct This<T: 'static + Send + Sync> {
    pub val : &'static mut T,
}

impl<T: 'static + Send + Sync>  FunctionParameter for This<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let this = unsafe{
            std::mem::transmute::<&mut T, &'static mut T>(source.read_native_this()?)
        };
        Result::Ok(Self{val: this})
    }
}

impl<T: 'static + Send + Sync> std::ops::Deref for This<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<'a, T: 'static + Send + Sync> FunctionParameter for Data<'a, T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let feetch = unsafe{
            std::mem::transmute::<legion::resource::FetchMut<T>, legion::resource::FetchMut<'a, T>>(source.read_system_data::<T>()?)
        };
        Result::Ok(Self{fetch: feetch})
    }
}

impl<'a, T: 'static + Send + Sync> std::ops::Deref for Data<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.fetch
    }
}

impl FunctionParameter for f32 {
    fn read<PS: ParametersSource>(source: &mut PS)-> Result<Self, PS::ErrorType> {
        source.read_float()
    }
}

impl FunctionParameter for i32 {
    fn read<PS: ParametersSource>(source: &mut PS)-> Result<Self, PS::ErrorType> {
        source.read_i32()
    }
}

impl FunctionParameter for usize {
    fn read<PS: ParametersSource>(source: &mut PS)-> Result<Self, PS::ErrorType> {
        source.read_i32().map(|x| x as usize)
    }
}

impl FunctionParameter for bool {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_bool()
    }
}

impl FunctionParameter for () {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        Result::Ok(())
    }
}

impl<T: FunctionParameter> FunctionParameter for Vec<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_all::<T>()
    }
}

impl FunctionParameter for String {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<String, PS::ErrorType> {
        source.read_formatted()
    }
}

impl<A: FunctionParameter, B: FunctionParameter> FunctionParameter for (A, B) {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let a = A::read(source)?;
        let b = B::read(source)?;
        Result::Ok((a,b))
    }
}

impl<A: FunctionParameter, B: FunctionParameter, C: FunctionParameter> FunctionParameter for (A, B, C) {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let a = A::read(source)?;
        let b = B::read(source)?;
        let c = C::read(source)?;
        Result::Ok((a, b, c))
    }
}
