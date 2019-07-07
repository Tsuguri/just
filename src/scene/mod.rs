use slotmap::SparseSecondaryMap as SecondaryMap;

mod game_object;


use game_object::GameObject;

struct Mesh;

struct Animator;

struct Audio;

struct Controller;

slotmap::new_key_type!(pub struct GameObjectId;);

type Map<T> = slotmap::HopSlotMap<GameObjectId, T>;
type Data<T> = SecondaryMap<GameObjectId, T>;

pub struct Scene {
    // at the same time indicates if object is active
    objects: Map<bool>,

    object_data: Data<GameObject>,
    renderables: Data<Mesh>,
    controllers: Data<Controller>,

    to_destroy: Vec<GameObjectId>,
}

enum GameObjectError {
    IdNotExisting,
}

impl Scene {
    pub fn set_parent(&mut self, obj: GameObjectId, new_parent: Option<GameObjectId>) -> Result<(), GameObjectError> {
        if !self.exists(obj) {
            return Result::Err(GameObjectError::IdNotExisting);
        }
        match new_parent {
            Some(x) => {
                if !self.exists(x) {
                    return Result::Err(GameObjectError::IdNotExisting);
                }
                self.object_data[x].children.push(obj);
            }
            None => (),
        }
        match self.object_data[obj].parent {
            None => (),
            Some(x) => {
                let index = self.object_data[x].children.iter().position(|y| *y == obj).unwrap();
                self.object_data[x].children.remove(index);
            }
        }
        self.object_data[obj].parent = new_parent;

        Result::Ok(())
    }
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            objects: Map::with_key(),
            object_data: Data::new(),
            renderables: Data::new(),
            controllers: Data::new(),
            to_destroy: vec![],
        }
    }
}

impl Scene {
    pub fn exists(&self, id: GameObjectId) -> bool {
        self.objects.contains_key(id)
    }

    pub fn create_game_object(&mut self) -> GameObjectId {
        let id = self.objects.insert(true);
        let go = GameObject::new(id);
        self.object_data.insert(id, go);
        id
    }

    pub fn remove_game_object(&mut self, id: GameObjectId) {
        let data = &self.object_data[id];
        for child in data.children.clone() {
            self.remove_game_object(child);
        }
        self.remove_single(id);
    }

    fn remove_single(&mut self, id: GameObjectId) {
        self.objects.remove(id);
        self.object_data.remove(id);
        self.renderables.remove(id);
        self.controllers.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut scene = Scene::new();
    }

    #[test]
    fn simple2() {
        let mut scene = Scene::new();
        let obj1 = scene.create_game_object();
        let obj2 = scene.create_game_object();
        scene.set_parent(obj1, Option::Some(obj2));

        assert_eq!(scene.object_data[obj1].parent, Option::Some(obj2));
        assert!(scene.object_data[obj2].children.contains(&obj1));

        scene.set_parent(obj1, Option::None);
        assert_eq!(scene.object_data[obj1].parent, Option::None);
        assert!(!scene.object_data[obj2].children.contains(&obj1));

    }

}
