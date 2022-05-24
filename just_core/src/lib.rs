pub mod game_object;
pub mod hierarchy;
pub mod math;
pub mod traits;
pub mod transform;

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

#[derive(Debug, Clone, Default)]
pub struct RenderableCreationQueue {
    pub queue: Vec<(ecs::entity::Entity, String)>,
}
