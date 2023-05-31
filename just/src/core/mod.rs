mod game;
mod parent_child_manipulation;
mod time;

use just_wgpu::winit::event::{ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent};
use just_wgpu::winit::event_loop::ControlFlow;
use time::TimeSystem;

use just_core::math::Vec2;
use just_core::{game_object, hierarchy};
use just_input::InputSystem;

use just_wgpu::winit;
use just_wgpu::RenderingSystem;
use winit::event_loop::EventLoop;

use just_core::ecs::prelude::*;

use just_assets::AssetSystem;

pub use game_object::GameObject;
pub use hierarchy::TransformHierarchy;

use game::GameLogic;

struct Animator;

struct Audio;

pub struct Engine {
    event_loop: Option<EventLoop<()>>,
    pub world: World,
}

#[derive(Debug)]
pub enum GameObjectError {
    IdNotExisting,
}

impl std::ops::Drop for Engine {
    fn drop(&mut self) {
        RenderingSystem::shut_down(&mut self.world);
        AssetSystem::cleanup(&mut self.world);
    }
}

impl Engine {
    pub fn new(res_path: &str) -> Self {
        let mut world = World::default();
        let event_loop = EventLoop::<()>::new();
        AssetSystem::initialize(&mut world, res_path);
        RenderingSystem::initialize(&mut world, &event_loop);
        InputSystem::initialize(&mut world);
        GameObject::initialize(&mut world);
        TimeSystem::initialize(&mut world);

        let mut eng = Engine {
            event_loop: Some(event_loop),
            world,
        };

        RenderingSystem::update(&mut eng.world);

        GameLogic::initialize(&mut eng.world);
        eng
    }

    pub fn run(mut self) {
        use just_input::InputEvent;
        use just_input::KeyboardState;
        use just_input::MouseState;

        let mut end_requested = false;
        let mut new_events = Vec::with_capacity(20);
        let event_loop = self.event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| {
            //*control_flow = ControlFlow::Poll;
            match event {
                Event::WindowEvent { window_id, ref event } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            end_requested = true;
                        }
                        WindowEvent::Resized(logical) => {
                            RenderingSystem::resize(&mut self.world, *logical);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            RenderingSystem::resize(&mut self.world, **new_inner_size);
                        }
                        WindowEvent::CursorMoved { position, .. } => {
                            let mut mouse_state = <Write<MouseState>>::fetch(&mut self.world.resources);

                            mouse_state.set_new_position([position.x as f32, position.y as f32]);
                            new_events.push(InputEvent::MouseMoved(Vec2::new(position.x as f32, position.y as f32)));
                        }
                        WindowEvent::MouseInput { state, button, .. } => {
                            let id: usize = match button {
                                MouseButton::Left => 0,
                                MouseButton::Right => 1,
                                MouseButton::Middle => 2,
                                MouseButton::Other(oth) => *oth as usize,
                            };
                            let mut mouse_state = <Write<MouseState>>::fetch(&mut self.world.resources);
                            mouse_state.set_button_state(id, *state == ElementState::Pressed);
                            match state {
                                ElementState::Pressed => new_events.push(InputEvent::MouseButtonPressed(id)),
                                ElementState::Released => new_events.push(InputEvent::MouseButtonReleased(id)),
                            }
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            // println!("pressed {:?}", input);

                            if input.virtual_keycode == Some(VirtualKeyCode::Escape) {
                                end_requested = true;
                            }
                            let elem = match input.virtual_keycode {
                                Some(key) => key,
                                None => return,
                            };
                            let kc = just_input::KeyCode::from_kc_enum(elem);
                            let mut keyboard_state = <Write<KeyboardState>>::fetch(&mut self.world.resources);
                            keyboard_state.set_button(kc, input.state == ElementState::Pressed);
                            match input.state {
                                ElementState::Pressed => new_events.push(InputEvent::KeyPressed(kc)),
                                ElementState::Released => new_events.push(InputEvent::KeyReleased(kc)),
                            }
                        }
                        _ => (),
                    }
                }
                Event::MainEventsCleared => {
                    RenderingSystem::maintain(&mut self.world);

                    let mut channel = <Write<just_input::InputChannel>>::fetch(&mut self.world.resources);
                    channel.drain_vec_write(&mut new_events);

                    if end_requested {
                        println!("end_requested");
                        *control_flow = ControlFlow::Exit;
                        return;
                    }

                    self.update();

                    let (mut keyboard_state, mut mouse_state) =
                        <(Write<KeyboardState>, Write<MouseState>)>::fetch(&mut self.world.resources);
                    keyboard_state.next_frame();
                    mouse_state.next_frame();
                }
                _ => {}
            }
        });
    }

    fn update(&mut self) {
        TimeSystem::update(&mut self.world);
        AssetSystem::update(&mut self.world);
        RenderingSystem::update(&mut self.world);

        GameLogic::update(&mut self.world);

        GameObject::remove_marked(&mut self.world);
    }
}

impl Engine {
    pub fn exists(&self, id: Entity) -> bool {
        self.world.is_alive(id)
    }

    pub fn create_game_object(&mut self) -> Entity {
        GameObject::create_empty(&mut self.world)
    }
}
