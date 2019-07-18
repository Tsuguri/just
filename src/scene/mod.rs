use slotmap::SparseSecondaryMap as SecondaryMap;
use std::mem::ManuallyDrop;

mod game_object;
pub mod scripting;
mod math;
mod parent_child_manipulation;
mod transform;
mod components;
pub mod traits;

pub use traits::*;

use crate::input;
use game_object::GameObject;
use scripting::{JsScript, JsScriptEngine};
#[cfg(test)]
use crate::scene::scripting::test_scripting::MockScriptEngine;

pub struct Mesh {}

struct Animator;

struct Audio;

slotmap::new_key_type!(pub struct GameObjectId;);

type Map<T> = slotmap::HopSlotMap<GameObjectId, T>;
type Data<T> = SecondaryMap<GameObjectId, T>;

pub struct Engine<E: ScriptingEngine, HW: Hardware> {
    // at the same time indicates if object is active
    pub objects: Map<bool>,

    pub object_data: Data<GameObject>,
    pub renderables: Data<Mesh>,
    controllers: Data<JsScript>,

    to_destroy: Vec<GameObjectId>,

    scripting_engine: E,
    pub resources: HW::RM,
    pub hardware: HW,
    renderer: HW::Renderer,
    keyboard: input::KeyboardState,
    mouse: input::MouseState,

}

pub type JsEngine = Engine<JsScriptEngine, super::graphics::Hardware>;

#[cfg(test)]
pub type MockEngine = Engine<MockScriptEngine, crate::graphics::test_resources::MockHardware>;


#[derive(Debug)]
pub enum GameObjectError {
    IdNotExisting,
}

impl<E: ScriptingEngine, HW: Hardware> Engine<E, HW> {
    //type HWA = i32;
    //use <HW as traits::Hardware> as HW;
    pub fn new(engine_config: &E::Config, hw_config: &HW::Config, rm_config: &<HW::RM as traits::ResourceManager>::Config) -> Self {
        let mut hardware = HW::create(hw_config);
        Engine {
            objects: Map::with_key(),
            object_data: Data::new(),
            renderables: Data::new(),
            controllers: Data::new(),
            to_destroy: vec![],
            scripting_engine: E::create(engine_config),
            resources: HW::RM::create(rm_config),
            renderer: HW::Renderer::create(&mut hardware),
            hardware,
            keyboard: Default::default(),
            mouse: Default::default(),
        }
    }
}

impl<E: ScriptingEngine, HW: Hardware> std::ops::Drop for Engine<E, HW> {
    fn drop(&mut self) {
        self.renderer.dispose(&mut self.hardware);
    }
}

impl JsEngine {
    pub fn run(&mut self) {
        use crate::input::*;
        self.hardware.event_loop.poll_events(|_| ());

        //let mut graph = crate::graphics::fill_render_graph(&mut self.hardware);


        loop {
            let inputs = UserInput::poll_events_loop(&mut self.hardware.event_loop, &mut self.keyboard, &mut self.mouse);
            if inputs.end_requested {
                break;
            }
            self.renderer.run(&mut self.hardware, &self.resources);
            //graph.run(&mut self.hardware.factory, &mut self.hardware.families, &());
        }

    }
}

impl<E: ScriptingEngine, HW: Hardware> Engine<E, HW> {
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
            if !self.exists(obj) {
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
    use crate::graphics::test_resources::MockResourceManager;

    #[test]
    fn simple() {
        let mrm = MockResourceManager {};
        let mut scene = MockEngine::new(&(1i32.into()), &1, &1);
    }
}
