use super::Engine;
use super::TransformHierarchy;
use just_core::ecs::prelude::Entity;

impl Engine {
    pub fn set_parent(&mut self, obj: Entity, new_parent: Option<Entity>) -> Result<(), ()> {
        TransformHierarchy::set_parent(&mut self.world, obj, new_parent)
    }
}
