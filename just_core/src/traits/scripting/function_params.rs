pub trait ParametersSource {
    type ErrorType: 'static + Send + Sync;

    fn read_float(&mut self) -> Result<f32, Self::ErrorType>;

    fn read_bool(&mut self) -> Result<bool, Self::ErrorType>;

    fn read_i32(&mut self) -> Result<i32, Self::ErrorType>;

    fn read_formatted(&mut self) -> Result<String, Self::ErrorType>;

    fn read_all<T: FunctionParameter>(&mut self) -> Result<Vec<T>, Self::ErrorType>;

    fn read_system_data<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<legion::resource::FetchMut<T>, Self::ErrorType>;

    fn read_world(&mut self) -> Result<&mut legion::world::World, Self::ErrorType>;

    fn read_native<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType>;

    fn read_native_this<T: 'static + Send + Sync + Sized>(&mut self) -> Result<&mut T, Self::ErrorType>;

    fn read_component<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<legion::borrow::RefMut<T>, Self::ErrorType>;

    fn read_component_this<T: 'static + Send + Sync + Sized>(
        &mut self,
    ) -> Result<legion::borrow::RefMut<T>, Self::ErrorType>;

    fn is_null(&self) -> bool;
}

pub trait FunctionParameter: Sized {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType>;
}

pub struct World<'a> {
    pub world: &'a mut legion::world::World,
}

pub struct Data<'a, T: 'static + Send + Sync> {
    pub fetch: legion::resource::FetchMut<'a, T>,
}

pub struct This<T: 'static + Send + Sync> {
    pub val: &'static mut T,
}

pub struct Component<T: 'static + Send + Sync> {
    pub val: &'static mut T,
}

pub struct ComponentThis<T: 'static + Send + Sync> {
    pub val: &'static mut T,
}

impl<T: 'static + Send + Sync> FunctionParameter for Component<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let component = unsafe { std::mem::transmute::<&mut T, &'static mut T>(&mut *source.read_component()?) };
        Result::Ok(Self { val: component })
    }
}

impl<T: 'static + Send + Sync> std::ops::Deref for Component<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T: 'static + Send + Sync> std::ops::DerefMut for Component<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<T: 'static + Send + Sync> FunctionParameter for ComponentThis<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let component = unsafe { std::mem::transmute::<&mut T, &'static mut T>(&mut *source.read_component_this()?) };
        Result::Ok(Self { val: component })
    }
}

impl<T: 'static + Send + Sync> std::ops::Deref for ComponentThis<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl<T: 'static + Send + Sync> std::ops::DerefMut for ComponentThis<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl<'a> FunctionParameter for World<'a> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        Result::Ok(Self {
            world: unsafe {
                std::mem::transmute::<&mut legion::world::World, &'static mut legion::world::World>(
                    source.read_world()?,
                )
            },
        })
    }
}

impl<'a> std::ops::Deref for World<'a> {
    type Target = legion::world::World;
    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl<'a> std::ops::DerefMut for World<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

impl<T: FunctionParameter> FunctionParameter for Option<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        if source.is_null() {
            Result::Ok(None)
        } else {
            Result::Ok(Some(T::read(source)?))
        }
    }
}

impl<T: 'static + Send + Sync> FunctionParameter for This<T> {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let this = unsafe { std::mem::transmute::<&mut T, &'static mut T>(source.read_native_this()?) };
        Result::Ok(Self { val: this })
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
        let feetch = unsafe {
            std::mem::transmute::<legion::resource::FetchMut<T>, legion::resource::FetchMut<'a, T>>(
                source.read_system_data::<T>()?,
            )
        };
        Result::Ok(Self { fetch: feetch })
    }
}

impl<'a, T: 'static + Send + Sync> std::ops::Deref for Data<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.fetch
    }
}

impl FunctionParameter for f32 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_float()
    }
}

impl FunctionParameter for i32 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_i32()
    }
}

impl FunctionParameter for usize {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_i32().map(|x| x as usize)
    }
}

impl FunctionParameter for bool {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        source.read_bool()
    }
}

impl FunctionParameter for () {
    fn read<PS: ParametersSource>(_source: &mut PS) -> Result<Self, PS::ErrorType> {
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
        Result::Ok((a, b))
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

impl FunctionParameter for glam::Vec2 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl FunctionParameter for glam::Vec3 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl FunctionParameter for glam::Vec4 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl FunctionParameter for glam::Mat4 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl FunctionParameter for glam::Mat3 {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}

impl FunctionParameter for glam::Quat {
    fn read<PS: ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}
