use slotmap::SparseSecondaryMap as SecondaryMap;

mod game_object;
mod scripting;
mod math;
mod parent_child_manipulation;
mod transform;
mod components;

use game_object::GameObject;
use scripting::{JsScript, JsEngine, ScriptingEngine, Controller};
#[cfg(test)]
use crate::scene::scripting::test_scripting::MockEngine;

struct Mesh;

struct Animator;

struct Audio;

slotmap::new_key_type!(pub struct GameObjectId;);

type Map<T> = slotmap::HopSlotMap<GameObjectId, T>;
type Data<T> = SecondaryMap<GameObjectId, T>;

pub struct Scene<E: ScriptingEngine> {
    // at the same time indicates if object is active
    objects: Map<bool>,

    object_data: Data<GameObject>,
    renderables: Data<Mesh>,
    controllers: Data<JsScript>,

    to_destroy: Vec<GameObjectId>,

    scripting_engine: E,
}

pub type JsScene = Scene<JsEngine>;

#[cfg(test)]
pub type MockScene = Scene<MockEngine>;


#[derive(Debug)]
pub enum GameObjectError {
    IdNotExisting,
}

impl<E: ScriptingEngine> Scene<E> {
    pub fn new(engine_config: &E::Config) -> Self {
        Scene {
            objects: Map::with_key(),
            object_data: Data::new(),
            renderables: Data::new(),
            controllers: Data::new(),
            to_destroy: vec![],
            scripting_engine: E::create(engine_config)
        }
    }
}

impl<E: ScriptingEngine> Scene<E> {
    pub fn exists(&self, id: GameObjectId) -> bool {
        self.objects.contains_key(id)
    }

    pub fn create_game_object(&mut self) -> GameObjectId {
        let id = self.objects.insert(true);
        let go = GameObject::new(id);
        self.object_data.insert(id, go);
        id
    }

    pub fn mark_to_remove(&mut self, id: GameObjectId) {
        self.to_destroy.push(id);
    }

    pub fn remove_marked(&mut self) {
        let objects = std::mem::replace(&mut self.to_destroy, vec![]);
        for obj in objects.into_iter() {
            // might have been removed as child of other object
            if !self.exists(obj){
                continue;
            }
            self.remove_game_object(obj);
        }

    }

    fn remove_game_object(&mut self, id: GameObjectId) {
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
        let mut scene = MockScene::new(&(1i32.into()));
    }
}
