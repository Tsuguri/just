pub mod game_object;
pub mod hierarchy;
pub mod math;
pub mod transform;

pub use legion as ecs;
pub use nalgebra_glm as glm;
pub use shrev;

pub use serde;

#[derive(Debug, Copy, Clone)]
pub struct GameObjectData {
    pub id: ecs::entity::Entity,
}
