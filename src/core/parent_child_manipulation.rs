use super::Engine;
use super::GameObjectError;
use crate::traits::*;
use legion::prelude::Entity;

impl<E: ScriptingEngine, HW: Hardware> Engine<E, HW> {
    pub fn set_parent(&mut self, obj: Entity, new_parent: Option<Entity>) -> Result<(), ()> {
        self.world.set_parent(obj, new_parent)
    }
}

#[cfg(test)]
mod tests {
    use super::super::MockEngine;
    #[test]
    fn set_parent() {
        let mut scene = MockEngine::mock();
        let obj1 = scene.create_game_object();
        let obj2 = scene.create_game_object();
        scene.set_parent(obj1, Option::Some(obj2)).unwrap();

        assert_eq!(scene.world.object_data[obj1].parent, Option::Some(obj2));
        assert!(scene.world.object_data[obj2].children.contains(&obj1));

        scene.set_parent(obj1, Option::None).unwrap();
        assert_eq!(scene.world.object_data[obj1].parent, Option::None);
        assert!(!scene.world.object_data[obj2].children.contains(&obj1));

    }

    #[test]
    fn removing_objects() {
        let mut scene = MockEngine::mock();
        let obj1 = scene.create_game_object();
        let obj2 = scene.create_game_object();
        let obj3 = scene.create_game_object();

        assert!(scene.exists(obj1));
        assert!(scene.exists(obj2));
        assert!(scene.exists(obj3));
        scene.set_parent(obj2, Option::Some(obj3)).unwrap();

        scene.world.remove_game_object(obj3, &mut scene.controllers);

        assert!(scene.exists(obj1));
        assert!(!scene.exists(obj2));
        assert!(!scene.exists(obj3));
    }

}
