pub mod math;
pub mod traits;

pub use nalgebra_glm as glm;
pub use legion as ecs;
pub use shrev;

pub use serde;

#[derive(Debug, Copy, Clone)]
pub struct GameObjectData {
    pub id: ecs::entity::Entity,
}

impl traits::scripting::FunctionResult for GameObjectData {}
impl traits::scripting::FunctionParameter for GameObjectData {
    fn read<PS: traits::scripting::ParametersSource>(source: &mut PS) -> Result<Self, PS::ErrorType> {
        let nat = source.read_native()?;
        Result::Ok(*nat)
    }
}
