mod parent_child_manipulation;
mod time;

use time::TimeSystem;

use just_input::InputSystem;
use just_core::math::{MathApi, Vec2};
use just_core::{
    game_object,
    hierarchy,
    transform,
};
use crate::apis::ConsoleApi;
use crate::apis::RenderableApi;
use crate::apis::WorldApi;
use crate::apis::TransformApi;
use just_core::traits::scripting::ScriptApiRegistry;

use just_assets::{AssetStorage, Handle};
use just_wgpu::Mesh;
use just_wgpu::Texture;

use just_core::traits::scripting::ScriptingEngine;
use just_core::ecs::prelude::*;

#[cfg(test)]
use crate::scripting::test_scripting::MockScriptEngine;
use std::sync::Arc;
use just_js::JsScriptEngine;
use just_assets::AssetSystem;
use just_wgpu::winit::event_loop::EventLoop;

pub use game_object::GameObject;
pub use hierarchy::TransformHierarchy;

struct Animator;

struct Audio;

use just_wgpu::RenderingSystem;

pub struct Engine<E: ScriptingEngine> {
    event_loop: Option<EventLoop<()>>,
    pub world: World,

    scripting_engine: E,
}

pub type JsEngine = Engine<JsScriptEngine>;

#[cfg(test)]
pub type MockEngine = Engine<MockScriptEngine>;

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
        let event_loop = EventLoop::new();
        AssetSystem::initialize(&mut world, res_path);

        RenderingSystem::initialize(&mut world, &event_loop);
        InputSystem::initialize(&mut world);
        GameObject::initialize(&mut world);
        TimeSystem::initialize(&mut world);


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
            event_loop: Some(event_loop),
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
        RenderingSystem::shut_down(&mut self.world);
        AssetSystem::cleanup(&mut self.world);
    }
}

impl JsEngine {
    pub fn run(mut self) {
        use just_wgpu::winit::event_loop::ControlFlow;
        use just_wgpu::winit::event::{Event, VirtualKeyCode, WindowEvent, MouseButton, ElementState};
        use just_input::MouseState;
        use just_input::KeyboardState;
        use just_input::InputEvent;

        let mut end_requested = false;
        let mut new_frame_size = None;
        let mut new_events = Vec::with_capacity(20);
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow|{

            //*control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    end_requested = true;
                },
                Event::WindowEvent {
                    event: WindowEvent::Resized(logical),
                    ..
                } => {
                    new_frame_size = Some((logical.width, logical.height));
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let mut mouse_state =
                        <Write<MouseState>>::fetch(
                            &mut self.world.resources,
                        );

                    mouse_state.set_new_position([position.x as f32, position.y as f32]);
                    new_events.push(InputEvent::MouseMoved(Vec2::new(
                        position.x as f32,
                        position.y as f32,
                    )));
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    let id: usize = match button {
                        MouseButton::Left => 0,
                        MouseButton::Right => 1,
                        MouseButton::Middle => 2,
                        MouseButton::Other(oth) => oth as usize,
                    };
                    let mut mouse_state =
                        <Write<MouseState>>::fetch(
                            &mut self.world.resources,
                        );
                    mouse_state.set_button_state(id, state == ElementState::Pressed);
                    match state {
                        ElementState::Pressed => new_events.push(InputEvent::MouseButtonPressed(id)),
                        ElementState::Released => new_events.push(InputEvent::MouseButtonReleased(id)),
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    //println!("pressed {:?}", input);

                    if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                        end_requested = true;
                    }
                    let elem = match input.virtual_keycode {
                        Some(key) => key,
                        None => return,
                    };
                    let kc = just_input::KeyCode::from_kc_enum(elem);
                    let mut keyboard_state =
                        <Write<KeyboardState>>::fetch(
                            &mut self.world.resources,
                        );
                    keyboard_state.set_button(kc, input.state == ElementState::Pressed);
                    match input.state {
                        ElementState::Pressed => new_events.push(InputEvent::KeyPressed(kc)),
                        ElementState::Released => new_events.push(InputEvent::KeyReleased(kc)),
                    }
                },
                Event::MainEventsCleared => {
                    //*control_flow = ControlFlow::Poll;
                    RenderingSystem::maintain(&mut self.world);

                    let mut channel =
                        <Write<just_input::InputChannel>>::fetch(
                            &mut self.world.resources,
                        );
                    channel.drain_vec_write(&mut new_events);


                    if end_requested {
                        println!("end_requested");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    TimeSystem::update(&mut self.world);
                    AssetSystem::update(&mut self.world);

                    self.update_scripts();
                    RenderingSystem::update(&mut self.world);

                    GameObject::remove_marked(&mut self.world);

                    let (mut keyboard_state, mut mouse_state) =
                        <(Write<KeyboardState>, Write<MouseState>)>::fetch(
                            &mut self.world.resources,
                        );
                    keyboard_state.next_frame();
                    mouse_state.next_frame();
                }
                _ => (),
            }

        });
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
        let res2 = self.world.resources.get::<AssetStorage<Mesh>>().unwrap();
        let res = self.world.resources.get::<AssetStorage<Texture>>().unwrap();
        let mesh_handle = res2.get_handle(mesh).unwrap();
        let tex = match tex {
            None => None,
            Some(tex_name) => {
                let tex_res = res.get_handle(tex_name).unwrap();
                Some(tex_res)
            }
        };
        let mesh = just_wgpu::Renderable {
            texture_handle: tex,
            mesh_handle: Some(mesh_handle),
        };
        drop(res);
        drop(res2);

        //Renderable::add_tex_renderable(&mut self.world, id, mesh);
    }

    pub fn add_script(&mut self, entity_id: Entity, typ: &str) {
        self
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
