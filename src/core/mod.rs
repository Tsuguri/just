use slotmap::SparseSecondaryMap as SecondaryMap;

mod game_object;
mod parent_child_manipulation;
mod transform;
mod components;
use crate::traits::{
    Data,
    Map,
    World,
    Hardware,
    RenderingData,
    MeshId,
    TextureId,
    GameObjectId,
    ScriptingEngine,
    ResourceManager,
    Renderer,
};

use crate::math::*;


use crate::input;
use game_object::GameObject;
use crate::scripting::JsScriptEngine;
#[cfg(test)]
use crate::scripting::test_scripting::MockScriptEngine;
use std::sync::Arc;

pub struct Mesh {
    pub mesh_id: MeshId,
    pub texture_id: Option<TextureId>,
}

struct Animator;

struct Audio;



pub struct WorldData {
    pub object_data: Data<GameObject>,
    pub renderables: Data<Mesh>,

}

impl World for WorldData {
    fn set_local_pos(&mut self, id: GameObjectId, new_position: Vec3) -> Result<(), ()>{
       self.set_local_position(id, new_position);
        Result::Ok(())
    }
    fn get_local_pos(&self, id: GameObjectId) -> Result<Vec3, ()>{
        Result::Ok(self.get_local_position(id))


    }
}

unsafe impl Send for WorldData{}
unsafe impl Sync for WorldData{}

impl WorldData {
    pub fn add_renderable(&mut self, id: GameObjectId,mesh: Mesh){
        self.renderables.insert(id, mesh);
    }
}

use crate::math::Matrix;

impl RenderingData for WorldData {
    fn get_projection_matrix(&self) -> Matrix {
//        let tmp2 = nalgebra_glm::perspective
        let mut temp = nalgebra_glm::perspective_lh_zo(
            256.0f32 / 108.0, f32::to_radians(45.0f32), 0.1f32, 100.0f32);
        temp[(1, 1)] *= -1.0;
        temp
    }

    fn get_view_matrix(&self) -> Matrix {
        nalgebra_glm::translation(&nalgebra_glm::vec3(1.0f32, -2.5, 10.0))
    }

    fn get_renderables(
        &self,
        buffer: Option<Vec<(MeshId, Option<TextureId>, Matrix)>>
    ) -> Vec<(MeshId, Option<TextureId>, Matrix)> {
        let mut buf = match buffer {
            Some(mut vec) => {
                if vec.len() < self.renderables.len() {
                    vec.reserve(self.renderables.len() - vec.len());
                }
                vec.clear();
                vec
            }
            None => Vec::with_capacity(self.renderables.len()),
        };

        //fill here
        for renderable in &self.renderables {

            let mat = self.object_data[renderable.0].get_global_matrix(self);

            buf.push((renderable.1.mesh_id, renderable.1.texture_id, mat));
        }
        buf
    }
}

pub struct Engine<E: ScriptingEngine, HW: Hardware + 'static> {
    // at the same time indicates if object is active
    pub objects: Map<bool>,
    pub world: WorldData,

    controllers: Data<E::Controller>,

    to_destroy: Vec<GameObjectId>,

    scripting_engine: E,
    pub resources: Arc<HW::RM>,
    pub hardware: HW,
    renderer: HW::Renderer,
    keyboard: input::KeyboardState,
    mouse: input::MouseState,

}

type Hw =super::graphics::Hardware<rendy::vulkan::Backend>;
pub type JsEngine = Engine<JsScriptEngine, Hw>;

#[cfg(test)]
pub type MockEngine = Engine<MockScriptEngine, crate::graphics::test_resources::MockHardware>;


#[derive(Debug)]
pub enum GameObjectError {
    IdNotExisting,
}

impl<E: ScriptingEngine, HW: Hardware + 'static> Engine<E, HW>
    where <HW as Hardware>::RM: Send + Sync
{
    //type HWA = i32;
    //use <HW as traits::Hardware> as HW;
    pub fn new(engine_config: &E::Config, hw_config: &HW::Config, rm_config: &<HW::RM as ResourceManager<HW>>::Config) -> Self {
        let mut hardware = HW::create(hw_config);
        let resources = Arc::new(HW::RM::create(rm_config, &mut hardware));
        let world = WorldData {
            object_data: Data::new(),
            renderables: Data::new(),
        };
        let renderer = HW::Renderer::create(&mut hardware, &world, resources.clone());
        let mut eng =Engine {
            objects: Map::with_key(),
            world,
            controllers: Data::new(),
            to_destroy: vec![],
            scripting_engine: E::create(engine_config),
            resources,
            renderer,
            hardware,
            keyboard: Default::default(),
            mouse: Default::default(),
        };
//        eng.scripting_engine.set_world_data(&mut eng.world);
        eng
    }

    fn update_scripts(&mut self, time: f64) {
        use std::ops::Deref as _;

        let rm = self.resources.deref();
        self.scripting_engine.update(
            &mut self.world,
            &mut self.controllers,
            rm,
            &self.keyboard,
            &self.mouse,
            time,
        );
    }
}

impl<E: ScriptingEngine, HW: Hardware + 'static> std::ops::Drop for Engine<E, HW> {
    fn drop(&mut self) {
        self.renderer.dispose(&mut self.hardware, &self.world);
    }
}

impl JsEngine {
    pub fn run(&mut self) {
        use crate::input::*;
        let start = std::time::Instant::now();

        loop {
            self.hardware.factory.maintain(&mut self.hardware.families);
            let inputs = UserInput::poll_events_loop(&mut self.hardware.event_loop, &mut self.keyboard, &mut self.mouse);
            if inputs.end_requested {
                break;
            }
            let duration = start.elapsed();

            let elapsed = duration.as_secs() as f64
                + duration.subsec_nanos() as f64 * 1e-9;

            self.update_scripts(elapsed);
            self.renderer.run(&mut self.hardware, &self.resources, &self.world);
            if elapsed > 5.0 {
                return;
            }
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
        self.world.object_data.insert(id, go);
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
        let data = &self.world.object_data[id];
        for child in data.children.clone() {
            self.remove_game_object(child);
        }
        self.remove_single(id);
    }

    fn remove_single(&mut self, id: GameObjectId) {
        self.objects.remove(id);
        self.world.object_data.remove(id);
        self.world.renderables.remove(id);
        self.controllers.remove(id);
    }

    pub fn add_renderable(&mut self, id: GameObjectId, mesh: &str) {
        let mesh = self.resources.get_mesh(mesh).unwrap();
        let mesh = Mesh{
            mesh_id: mesh,
            texture_id: None,
        };

        self.world.add_renderable(id, mesh);
    }

    pub fn add_script(&mut self, id: GameObjectId, typ: &str) {
        let obj = self.scripting_engine.create_script(id, typ);

        self.controllers.insert(id, obj);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::test_resources::MockResourceManager;

    #[test]
    fn simple() {
        let _mrm = MockResourceManager {};
        let _scene = MockEngine::new(&(1i32.into()), &1, &1);
    }
}
