use game_object::GameObject;
use hierarchy::TransformHierarchy;
use just_core::ecs::prelude::*;
use just_core::math::{Quat, Vec2, Vec3};
use just_core::{game_object, hierarchy};
use just_input::{InputChannel, InputEvent, InputReader, KeyCode, KeyboardState, MouseState};
use just_wgpu::{RenderingSystem, ScreenData, Ui};
use std::f32::consts::PI;

struct GameState {
    input_reader: InputReader,
    player: Entity,
    camera_lookat: Entity,
    pan: Option<PanningAction>,
}

#[derive(Default)]
struct PanningAction {
    start: Vec2,   // screen coordinates
    current: Vec2, // screen coordinates
}

#[derive(Default)]
struct Input {
    // continuous
    move_left: bool,
    move_right: bool,
    move_up: bool,
    move_down: bool,
    // actions: true in first frame after button pressed
    open_inventory: bool,
}

pub struct GameLogic;

impl GameLogic {
    pub fn initialize(world: &mut World) {
        let reader = {
            let mut channel = world.resources.get_mut::<InputChannel>().unwrap();
            channel.register_reader()
        };

        let id = GameObject::create_empty(world);
        GameObject::set_name(world, id, "duh".to_owned());
        //TransformHierarchy::set_local_position(&mut self.world, id, Vec3::new(10.0, 20.0, 30.0));
        RenderingSystem::add_renderable(world, id, "cow1", "creature");

        let camera_lookat = GameObject::create_empty(world);

        let id2 = GameObject::create_empty(world);
        GameObject::set_name(world, id2, "duhesse".to_owned());
        RenderingSystem::add_renderable(world, id2, "floor", "grassland");
        TransformHierarchy::set_local_position(world, id2, Vec3::new(-20.0, -2.0, 20.0));
        TransformHierarchy::set_local_rotation(world, id2, Quat::from_rotation_y(-PI / 4.0));
        //TransformHierarchy::set_local_scale(world, id2, Vec3::new(10.0, 10.0, 10.0));

        {
            let mut screen_data = world.resources.get_mut::<just_wgpu::ScreenData>().unwrap();
            screen_data.camera.position = Vec3::new(0.0, 1.0, 2.0);
            //camera_data.rotation = Quat::from_euler(just_core::glam::EulerRot::XYZ, -PI / 4.0, PI / 6.0, 0.0);
        }
        world.resources.insert(GameState {
            input_reader: reader,
            player: id,
            camera_lookat,
            pan: None,
        });
    }
    pub fn update(world: &mut World) {
        let player_input = {
            let (ui, keyboard_state, mouse_state, mut channel, mut state, screen) =
                <(
                    Read<Ui>,
                    Read<KeyboardState>,
                    Read<MouseState>,
                    Write<InputChannel>,
                    Write<GameState>,
                    Read<ScreenData>,
                )>::fetch(&mut world.resources);

            let mut player_input = Input {
                move_left: keyboard_state.is_button_down(KeyCode::A),
                move_right: keyboard_state.is_button_down(KeyCode::D),
                move_up: keyboard_state.is_button_down(KeyCode::W),
                move_down: keyboard_state.is_button_down(KeyCode::S),
                ..Default::default()
            };

            for event in ui.filter_input(channel.read(&mut state.input_reader)) {
                // will do something eventually
                match event {
                    InputEvent::KeyPressed(KeyCode::I) => player_input.open_inventory = true,
                    InputEvent::MouseButtonPressed(0) => {
                        println!("not ignoring press");
                        //mouse_state.current_posit
                    }
                    InputEvent::MouseButtonPressed(1) => {
                        let mouse_position = screen
                            .viewport
                            .viewport_pos_to_screen_space(&mouse_state.current_position.into());
                        println!("pan start");
                        state.pan = Some(PanningAction {
                            start: mouse_position,
                            current: mouse_position,
                        })
                    }
                    InputEvent::MouseButtonReleased(1) => {
                        println!("pan end");
                    }
                    InputEvent::MouseMoved(pos) => match &mut state.pan {
                        None => (),
                        Some(current_pan) => {
                            current_pan.current = screen.viewport.viewport_pos_to_screen_space(pos);
                        }
                    },
                    _ => {}
                }
            }
            player_input
        };

        Self::handle_player_input(world, player_input);

        {
            let (player, camera) = {
                let state = world.resources.get::<GameState>().unwrap();
                (state.player, state.camera_lookat)
            };

            let lookat = TransformHierarchy::get_local_position(world, camera);
            let player = TransformHierarchy::get_local_position(world, player);
            let new_lookat = lookat * 0.91 + player * 0.09;
            TransformHierarchy::set_local_position(world, camera, new_lookat);
            {
                let mut screen_data = world.resources.get_mut::<just_wgpu::ScreenData>().unwrap();
                screen_data.camera.position =
                    new_lookat + Vec3::new(10.0, 10.0 * 2.0f32.sqrt() * 4.0 / 3.0, -10.0) * 0.5;
                // camera_data.position = new_lookat + Vec3::new(0.0, 1.0, 2.0);
                let cam_rot = just_core::glam::Mat4::look_at_lh(
                    screen_data.camera.position,
                    new_lookat,
                    Vec3::new(0.0, 1.0, 0.0),
                );
                screen_data.camera.rotation = Quat::from_mat4(&cam_rot);
            }
        }
    }

    pub fn cleanup(world: &mut World) {}

    fn handle_player_input(world: &mut World, input: Input) {
        let state = world.resources.get::<GameState>().unwrap();
        let id = state.player;
        let pos = TransformHierarchy::get_local_position(world, id);

        let vertical = if input.move_up { 1.0 } else { -1.0 } + if input.move_down { -1.0 } else { 1.0 };
        let horizontal = if input.move_right { 1.0 } else { -1.0 } + if input.move_left { -1.0 } else { 1.0 };

        let new_pos = pos + Vec3::new(horizontal, 0.0, vertical) * 0.05;

        drop(state);
        TransformHierarchy::set_local_position(world, id, new_pos);
    }
}
