pub mod game_object;
pub mod hierarchy;
pub mod math;
pub mod transform;

pub use legion as ecs;
pub use shrev;

pub use serde;

pub use glam;

#[derive(Debug, Copy, Clone)]
pub struct GameObjectData {
    pub id: ecs::entity::Entity,
}

#[derive(Debug, Clone, Default)]
pub struct RenderableCreationQueue {
    pub queue: Vec<(ecs::entity::Entity, String, Option<String>)>,
}
