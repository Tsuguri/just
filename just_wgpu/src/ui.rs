use std::ops::{Deref, DerefMut};

use egui::{Event, FullOutput, Modifiers, Pos2, RawInput};
use just_core::ecs::prelude::*;
use just_input::{InputChannel, InputEvent, InputEvents, InputReader, KeyboardState, MouseState};

pub struct UiState {
    message: String,
}

pub struct Ui {
    ctx: egui::Context,
    reader: InputReader,
}

impl Deref for Ui {
    type Target = egui::Context;
    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl DerefMut for Ui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

impl Ui {
    pub fn initialize(world: &mut World) {
        let reader = {
            let mut channel = world.resources.get_mut::<InputChannel>().unwrap();
            channel.register_reader()
        };
        world.resources.insert(UiState {
            message: "default".to_owned(),
        });
        world.resources.insert(Self {
            ctx: egui::Context::default(),
            reader,
        });
    }
    pub fn filter_input<'a>(&self, events: InputEvents<'a>) -> impl Iterator<Item = &'a InputEvent> {
        let wants_keyboard = self.wants_keyboard_input();
        let wants_mouse = self.wants_pointer_input();
        events.filter(move |ev| match ev {
            InputEvent::KeyPressed(_) => !wants_keyboard,
            InputEvent::KeyReleased(_) => true,
            InputEvent::MouseButtonPressed(_) => !wants_mouse,
            InputEvent::MouseButtonReleased(_) => true, //always pass button release
            InputEvent::MouseMoved(_) => true,
        })
    }

    pub fn update(world: &mut World) -> FullOutput {
        let (mut ui, mut state, channel, keyboard, mouse) = <(
            Write<Ui>,
            Write<UiState>,
            Read<InputChannel>,
            Read<KeyboardState>,
            Read<MouseState>,
        )>::fetch(&mut world.resources);
        let mut raw_input = RawInput::default();
        for event in channel.read(&mut ui.reader) {
            match event {
                InputEvent::MouseMoved(pos) => {
                    raw_input.events.push(Event::PointerMoved(Pos2::new(pos.x, pos.y)));
                }
                InputEvent::MouseButtonPressed(_) => raw_input.events.push(Event::PointerButton {
                    pos: Pos2::new(mouse.current_position[0], mouse.current_position[1]),
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                }),
                InputEvent::MouseButtonReleased(_) => raw_input.events.push(Event::PointerButton {
                    pos: Pos2::new(mouse.current_position[0], mouse.current_position[1]),
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                }),
                _ => {}
            }
        }
        ui.run(raw_input, |ctx| {
            egui::SidePanel::left("left panel").show(ctx, |ui| {
                ui.label("hello");
                if ui.button(&state.message).clicked() {
                    println!("clicked button");
                }
            });
        })
    }
}
