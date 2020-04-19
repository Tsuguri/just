use legion::prelude::*;
use std::sync::Arc;
use crate::traits::{ResourceProvider, TextureId};
use crate::math::*;


pub struct UiSystem {
    resources: Arc<dyn ResourceProvider>,
    pub buttons: Vec<Button>,
}

pub struct Button {
    pub position: Vec2,
    pub size: Vec2,
    pub texture: TextureId,
}



impl UiSystem {
    pub fn initialize(world: &mut World, resources: Arc<dyn ResourceProvider>) {
        let buttons = vec![
            Button {
                position: Vec2::new(100.0f32, 50.0f32),
                size: Vec2::new(200.0f32, 100.0f32),
                texture: resources.get_texture("tex1").unwrap(),
            },
            Button {
                position: Vec2::new(2560.0f32 - 100.0f32, 50.0f32),
                size: Vec2::new(200.0f32, 100.0f32),
                texture: resources.get_texture("tex1").unwrap(),
            },
            Button {
                position: Vec2::new(100.0f32, 1080.0f32 - 50.0f32),
                size: Vec2::new(200.0f32, 100.0f32),
                texture: resources.get_texture("tex1").unwrap(),
            },
            Button {
                position: Vec2::new(2560.0f32 - 100.0f32, 1080.0f32 - 50.0f32),
                size: Vec2::new(200.0f32, 100.0f32),
                texture: resources.get_texture("tex1").unwrap(),
            },
        ];
        let system = UiSystem {
            resources,
            buttons
        };
        world.resources.insert(system);
    }

    pub fn update(world: &mut World) {

    }

    pub fn shut_down(world: &mut World) {
        world.resources.remove::<UiSystem>();
    }
}
