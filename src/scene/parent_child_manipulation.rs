use super::Engine;
use super::GameObjectError;
use crate::scene::traits::*;

impl<E: ScriptingEngine, HW: Hardware> Engine<E, HW> {
    pub fn set_parent(&mut self, obj: GameObjectId, new_parent: Option<GameObjectId>) -> Result<(), GameObjectError> {
        if !self.exists(obj) {
            return Result::Err(GameObjectError::IdNotExisting);
        }
        match new_parent {
            Some(x) => {
                if !self.exists(x) {
                    return Result::Err(GameObjectError::IdNotExisting);
                }
                self.world.object_data[x].children.push(obj);
            }
            None => (),
        }
        match self.world.object_data[obj].parent {
            None => (),
            Some(x) => {
                let index = self.world.object_data[x].children.iter().position(|y| *y == obj).unwrap();
                self.world.object_data[x].children.remove(index);
            }
        }
        self.world.object_data[obj].parent = new_parent;
        self.world.object_data[obj].void_local_matrix(&self.world);

        Result::Ok(())
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

        scene.remove_game_object(obj3);

        assert!(scene.exists(obj1));
        assert!(!scene.exists(obj2));
        assert!(!scene.exists(obj3));
    }

}