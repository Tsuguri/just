mod colliders;
mod components;
mod game_object;
mod hierarchy;
mod parent_child_manipulation;
mod transform;
mod world_data;

use crate::traits::{
    Data, Hardware, MeshId, Renderer, ResourceManager, ResourceProvider, ScriptApiRegistry,
    ScriptingEngine, TextureId,
};
use crate::ui;
use legion::prelude::*;

use crate::input;
use crate::math::*;
use crate::scripting;
#[cfg(test)]
use crate::scripting::test_scripting::MockScriptEngine;
use crate::scripting::JsScriptEngine;
use std::sync::Arc;

use std::cell::RefCell;

use legion::prelude::*;

pub use game_object::GameObject;
pub use hierarchy::TransformHierarchy;
pub use world_data::Renderable;

struct Animator;

struct Audio;

pub struct Engine<E: ScriptingEngine, HW: Hardware + 'static> {
    pub world: World,

    scripting_engine: E,
    pub resources: Arc<HW::RM>,
    pub hardware: HW,
    renderer: HW::Renderer,
}

type Hw = super::graphics::Hardware<rendy::vulkan::Backend>;
pub type JsEngine = Engine<JsScriptEngine, Hw>;

#[cfg(test)]
pub type MockEngine = Engine<MockScriptEngine, crate::graphics::test_resources::MockHardware>;

#[derive(Debug)]
pub enum GameObjectError {
    IdNotExisting,
}

impl<E: ScriptingEngine + ScriptApiRegistry, HW: Hardware + 'static> Engine<E, HW>
where
    <HW as Hardware>::RM: Send + Sync,
{
    //type HWA = i32;
    //use <HW as traits::Hardware> as HW;
    pub fn new(
        engine_config: &E::Config,
        hw_config: &HW::Config,
        rm_config: &<HW::RM as ResourceManager<HW>>::Config,
    ) -> Self {
        let mut hardware = HW::create(hw_config);
        let resources = Arc::new(HW::RM::create(rm_config, &mut hardware));

        let mut world = World::default();
        world
            .resources
            .insert::<Arc<dyn ResourceProvider>>(resources.clone());
        input::UserInput::initialize(&mut world);
        GameObject::initialize(&mut world);
        ui::UiSystem::initialize(&mut world, resources.clone());
        TimeSystem::initialize(&mut world);

        let renderer = HW::Renderer::create(&mut hardware, &mut world, resources.clone());
        let mut scripting_engine = E::create(engine_config, &mut world);
        TimeSystem::register_api(&mut scripting_engine);
        let eng = Engine {
            world,
            scripting_engine,
            resources,
            renderer,
            hardware,
        };
        eng
    }

    fn update_scripts(&mut self) {
        self.scripting_engine.update(&mut self.world);
    }
}

impl<E: ScriptingEngine, HW: Hardware + 'static> std::ops::Drop for Engine<E, HW> {
    fn drop(&mut self) {
        self.renderer.dispose(&mut self.hardware, &self.world);
        ui::UiSystem::shut_down(&mut self.world);
    }
}

struct TimeData {
    start: std::time::Instant,
    elapsed: f32,
    dt: f32,
}

struct TimeSystem;

impl TimeSystem {
    pub fn initialize(world: &mut World) {
        let system = TimeData {
            start: std::time::Instant::now(),
            elapsed: 0f32,
            dt: 0.016f32,
        };
        world.resources.insert(system);
    }

    pub fn update(world: &mut World) {
        let mut sys = <(Write<TimeData>)>::fetch(&world.resources);
        let duration = sys.start.elapsed();

        let elapsed = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let dt = elapsed - sys.elapsed as f64;
        sys.dt = dt as f32;
        sys.elapsed = elapsed as f32;
    }

    pub fn register_api<SAR: crate::traits::ScriptApiRegistry>(sar: &mut SAR) {
        let nm = sar.register_namespace("Time", None);

        sar.register_static_property(
            "elapsed",
            Some(&nm),
            Some(|d: Data<TimeData>| d.fetch.elapsed),
            Some(|()| {}),
        );

        //sar.register_function()
    }
}

impl JsEngine {
    pub fn run(&mut self) {
        use crate::input::*;

        loop {
            self.hardware.factory.maintain(&mut self.hardware.families);

            let inputs =
                UserInput::poll_events_loop(&mut self.hardware.event_loop, &mut self.world);

            if inputs.end_requested {
                break;
            }

            TimeSystem::update(&mut self.world);

            self.update_scripts();
            ui::UiSystem::update(&mut self.world);
            self.renderer
                .run(&mut self.hardware, &self.resources, &self.world);

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
        let mesh = world_data::Renderable {
            mesh: Some(mesh),
            texture: tex,
        };

        Renderable::add_tex_renderable(&mut self.world, id, mesh);
    }

    pub fn add_script(&mut self, entity_id: Entity, typ: &str) {
        let obj = self
            .scripting_engine
            .create_script(entity_id, typ, &mut self.world);
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