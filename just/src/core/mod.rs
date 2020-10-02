mod game_object;
mod hierarchy;
mod parent_child_manipulation;
mod transform;
mod world_data;
mod time;

use time::TimeSystem;

use just_input::InputSystem;
use just_core::math::MathApi;
use crate::apis::ConsoleApi;
use crate::apis::RenderableApi;
use crate::apis::WorldApi;
use crate::apis::TransformApi;
use just_core::traits::scripting::ScriptApiRegistry;

use crate::traits::{
    ResourceProvider, 
};
use just_rendyocto::rendy;

use just_core::traits::scripting::ScriptingEngine;
use crate::ui;
use just_core::ecs::prelude::*;

#[cfg(test)]
use crate::scripting::test_scripting::MockScriptEngine;
use std::sync::Arc;
use just_js::JsScriptEngine;
use just_assets::AssetSystem;

pub use game_object::GameObject;
pub use hierarchy::TransformHierarchy;
pub use world_data::Renderable;

struct Animator;

struct Audio;

use super::graphics::{
    Hw,
    Res,
    Rd,
    RenderingSystem,
};

pub struct Engine<E: ScriptingEngine> {
    pub world: World,

    scripting_engine: E,
}

pub type JsEngine = Engine<JsScriptEngine>;

#[cfg(test)]
pub type MockEngine = Engine<MockScriptEngine, crate::graphics::test_resources::MockHardware>;

#[derive(Debug)]
pub enum GameObjectError {
    IdNotExisting,
}

impl<E: ScriptingEngine + ScriptApiRegistry> Engine<E>
{
    pub fn new(
        engine_config: &E::Config,
        res_path: &str,
    ) -> Self {
        let mut world = World::default();

        let resources = RenderingSystem::initialize(&mut world, res_path);

        InputSystem::initialize(&mut world);
        GameObject::initialize(&mut world);
        ui::UiSystem::initialize(&mut world, resources);
        TimeSystem::initialize(&mut world);
        AssetSystem::initialize(&mut world, res_path);


        let mut scripting_engine = E::create(engine_config, &mut world);
        TransformApi::register(&mut scripting_engine);
        WorldApi::register(&mut scripting_engine);
        TimeSystem::register_api(&mut scripting_engine);
        MathApi::register_api(&mut scripting_engine);
        ConsoleApi::register(&mut scripting_engine);
        AssetSystem::register_api(&mut scripting_engine);

        RenderableApi::register(&mut scripting_engine);
        InputSystem::register_api(&mut scripting_engine);
        let eng = Engine {
            world,
            scripting_engine,
        };
        eng
    }

    fn update_scripts(&mut self) {
        self.scripting_engine.update(&mut self.world);
    }
}

impl<E: ScriptingEngine> std::ops::Drop for Engine<E> {
    fn drop(&mut self) {
        ui::UiSystem::shut_down(&mut self.world);
        RenderingSystem::shut_down(&mut self.world);
    }
}

impl JsEngine {
    pub fn run(&mut self) {

        loop {
            RenderingSystem::maintain(&mut self.world);

            let mut rm =
                <Write<crate::graphics::RenderingManager>>::fetch(
                    &mut self.world.resources,
                );
            let inputs =
                just_input::InputSystem::poll_events(&mut rm.hardware.event_loop, &mut self.world);

            if inputs.end_requested {
                break;
            }

            TimeSystem::update(&mut self.world);
            AssetSystem::update(&mut self.world);

            self.update_scripts();
            ui::UiSystem::update(&mut self.world);
            RenderingSystem::run(&mut self.world);

            GameObject::remove_marked(&mut self.world);
        }
    }
}

impl<E: ScriptingEngine> Engine<E> {
    pub fn exists(&self, id: Entity) -> bool {
        self.world.is_alive(id)
    }

    pub fn create_game_object(&mut self) -> Entity {
        GameObject::create_empty(&mut self.world)
    }

    pub fn add_renderable(&mut self, id: Entity, mesh: &str, tex: Option<&str>) {
        let res = self.world.resources.get::<Arc<dyn ResourceProvider>>().unwrap();
        let mesh = res.get_mesh(mesh).unwrap();
        let tex = match tex {
            None => None,
            Some(tex_name) => {
                let tex_res = res.get_texture(tex_name).unwrap();
                Some(tex_res)
            }
        };
        let mesh = world_data::Renderable {
            mesh: Some(mesh),
            texture: tex,
        };
        drop(res);

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
