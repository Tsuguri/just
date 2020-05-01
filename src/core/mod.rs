mod game_object;
mod parent_child_manipulation;
mod transform;
mod components;
mod world_data;
mod colliders;
mod hierarchy;

use crate::traits::{
    Hardware,
    MeshId,
    TextureId,
    ScriptingEngine,
    ResourceManager,
    ResourceProvider,
    Renderer,
};
use crate::ui;
use legion::prelude::*;


use crate::input;
use crate::math::*;
use crate::scripting::JsScriptEngine;
#[cfg(test)]
use crate::scripting::test_scripting::MockScriptEngine;
use std::sync::Arc;

use std::cell::RefCell;

use legion::prelude::*;

pub use hierarchy::TransformHierarchy;
pub use game_object::GameObject;
pub use world_data::Mesh;


struct Animator;

struct Audio;

pub struct Engine<E: ScriptingEngine, HW: Hardware + 'static> {
    pub world: World,

    scripting_engine: E,
    pub resources: Arc<HW::RM>,
    pub hardware: HW,
    renderer: HW::Renderer,
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

        let mut world = World::default();
        world.resources.insert::<Arc<dyn ResourceProvider>>(resources.clone());
        input::UserInput::initialize(&mut world);
        GameObject::initialize(&mut world);
        ui::UiSystem::initialize(&mut world, resources.clone());

        let renderer = HW::Renderer::create(&mut hardware, &mut world, resources.clone());
        let scripting_engine = E::create(engine_config, &mut world);
        let eng =Engine {
            world,
            scripting_engine,
            resources,
            renderer,
            hardware,
        };
        eng
    }

    fn update_scripts(&mut self, time: f64) {

        self.scripting_engine.update(
            &mut self.world,
            time,
        );
    }
}

impl<E: ScriptingEngine, HW: Hardware + 'static> std::ops::Drop for Engine<E, HW> {
    fn drop(&mut self) {
        self.renderer.dispose(&mut self.hardware, &self.world);
        ui::UiSystem::shut_down(&mut self.world);
    }
}

impl JsEngine {
    pub fn run(&mut self) {
        use crate::input::*;
        let start = std::time::Instant::now();

        loop {
            self.hardware.factory.maintain(&mut self.hardware.families);

            let inputs = UserInput::poll_events_loop(&mut self.hardware.event_loop, &mut self.world);

            if inputs.end_requested {
                break;
            }
            let duration = start.elapsed();

            let elapsed = duration.as_secs() as f64
                + duration.subsec_nanos() as f64 * 1e-9;

            self.update_scripts(elapsed);
            ui::UiSystem::update(&mut self.world);
            self.renderer.run(&mut self.hardware, &self.resources, &self.world);

            GameObject::remove_marked(&mut self.world);
        }
    }
}

impl<E: ScriptingEngine, HW: Hardware> Engine<E, HW> {
    pub fn exists(&self, id: Entity) -> bool {
        self.world.is_alive(id)
    }

    pub fn create_game_object(&mut self) -> Entity {
        GameObject::create_empty(&mut self.world)
    }

    pub fn add_renderable(&mut self, id: Entity, mesh: &str, tex: Option<&str>) {
        let mesh = self.resources.get_mesh(mesh).unwrap();
        let tex = match tex {
            None => None,
            Some(tex_name) => {
                let tex_res = self.resources.get_texture(tex_name).unwrap();
                Some(tex_res)
            }
        };
        let mesh = world_data::Mesh{
            mesh_id: mesh,
            texture_id: tex,
        };

        Mesh::add_tex_renderable(&mut self.world, id, mesh);
    }

    pub fn add_script(&mut self, entity_id: Entity, typ: &str) {
        let obj = self.scripting_engine.create_script(entity_id, typ, &mut self.world);
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
