use legion::prelude::*;
use std::sync::Arc;
use std::collections::HashSet;
use crate::traits::{ResourceProvider, TextureId};
use crate::math::*;
use crate::input::{MouseState, InputEvent};
use stretch::style::*;
use stretch::node::*;



pub struct UiSystem {
    resources: Arc<dyn ResourceProvider>,
    pub layout: Stretch,
    root: Node,
    root_entity: Entity,
    pub mouse_over: HashSet<Entity>,
    reader_id: shrev::ReaderId<InputEvent>,
}

unsafe impl Send for UiSystem{}
unsafe impl Sync for UiSystem{}

impl UiSystem {
    pub fn create_node(&mut self, style: Style) -> Result<UiTransform, stretch::Error> {
        let node = self.layout.new_node(style, vec![])?;
        self.layout.add_child(self.root, node).unwrap();
        Ok(UiTransform {
            style,
            node,
            parent: Some(self.root_entity)
        })
    }
}

#[derive(Clone)]
pub struct UiTransform {
    pub style: Style,
    pub node: Node,
    pub parent: Option<Entity>,
}

pub struct UiClickable {
    pub callback: u32, // TODO: implement callbacks :)
}

pub enum UiRenderable {
    Rect(TextureId),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum UiEventType {
    Clicked,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct UiEvent {
    pub entity: Entity,
    pub event_type: UiEventType,
}

pub type UiEventChannel = shrev::EventChannel::<UiEvent>;


type InputEventChannel = shrev::EventChannel::<InputEvent>;

struct UiHierarchy {}

impl UiHierarchy {
    pub fn set_parent(world: &mut World, id: Entity, new_parent: Option<Entity>, _position: Option<u32>) -> Result<(),()> {
        if !world.is_alive(id) {
            return Result::Err(());
        }
        let node = world.get_component_mut::<UiTransform>(id).unwrap().node;
        let parent = Self::get_parent(world, id);
        match parent {
            None => (),
            Some(x) => {
                let parent_node = world.get_component_mut::<UiTransform>(x).unwrap().node;
                let mut system = world.resources.get_mut::<UiSystem>().unwrap();
                system.layout.remove_child(parent_node, node).unwrap();
            }
        }
        match new_parent {
            None => (),
            Some(x) => {
                let parent_node = world.get_component_mut::<UiTransform>(x).unwrap().node;
                let mut system = world.resources.get_mut::<UiSystem>().unwrap();
                system.layout.add_child(parent_node, node).unwrap();
            }
        }
        world.get_component_mut::<UiTransform>(id).unwrap().parent = new_parent;
        Ok(())
    }
    pub fn get_parent(world: &World, id: Entity) -> Option<Entity> {
        world.get_component::<UiTransform>(id).unwrap().parent
    }
}



impl UiSystem {
    pub fn initialize(world: &mut World, resources: Arc<dyn ResourceProvider>) {
        use stretch::geometry::Size;
        use stretch::style::*;

        let mut layout = Stretch::new();

        let root = layout.new_node(stretch::style::Style{
            size: Size{width: Dimension::Percent(1.0f32), height: Dimension::Percent(1.0f32)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..Default::default()
        }, vec![]).unwrap();

        let ent_id = world.insert(
            (),
            vec![()],
        ).to_vec();
        let root_entity = ent_id[0];
        let tex_id = resources.get_texture("tex1").unwrap();
        let mut input_channel = world.resources.get_mut::<InputEventChannel>().unwrap();
        let reader_id = input_channel.register_reader();
        drop(input_channel);

        let mut system = UiSystem {
            resources,
            layout,
            root,
            root_entity,
            mouse_over: HashSet::new(),
            reader_id
        };


        let entities = world.insert((), vec![
            (system.create_node(Style{size: Size{width: Dimension::Percent(1.0f32), height: Dimension::Points(200.0f32)}, ..Default::default()}).unwrap(),),
            (system.create_node(Style{size: Size{width: Dimension::Percent(1.0f32), height: Dimension::Percent(1.0f32)}, ..Default::default()}).unwrap(),),
            (system.create_node(Style{size: Size{width: Dimension::Percent(1.0f32), height: Dimension::Points(200.0f32)}, ..Default::default()}).unwrap(),),
            (system.create_node(Style{size: Size{width: Dimension::Percent(1.0f32), height: Dimension::Points(200.0f32)}, ..Default::default()}).unwrap(),),
        ]).to_vec();

        world.add_component::<UiRenderable>(entities[0], UiRenderable::Rect(tex_id));
        world.add_component::<UiRenderable>(entities[2], UiRenderable::Rect(tex_id));
        world.add_component::<UiClickable>(entities[0], UiClickable{callback: 0});
        world.add_component::<UiClickable>(entities[2], UiClickable{callback: 0});

        world.resources.insert(system);
        world.resources.insert(UiEventChannel::with_capacity(64))
    }

    fn check_mouse_press(position: [f32;2], ui: &UiSystem) {
        let pos = Vec2::new(position[0], 1080.0f32 - position[1]);
        println!("Checking ui input at: {:?}", pos);
        /*for (id, button) in ui.buttons.iter().enumerate() {
            let min = button.position - button.size * 0.5f32;
            let max = button.position + button.size * 0.5f32;
            println!("Button positions: min: {:?}, max: {:?}", min, max);
            if pos.x > min.x && pos.y > min.y  && pos.x < max.x && pos.y < max.y {
                println!("button pressed: {}", id);
            }
        }*/
    }

    pub fn update(world: &mut World) {
        // if ever focus is implemented - update here.

        let (
            mouse,
            mut ui,
            viewport_data,
            mut ui_events,
            mut input_events) = <(Read<MouseState>, Write<UiSystem>, Read<crate::graphics::ViewportData>, Write<UiEventChannel>, Write<InputEventChannel>)>::fetch(&world.resources);
        let clickable_query = <(Read<UiTransform>, Read<UiClickable>)>::query();

        let root = ui.root;

        ui.layout.compute_layout(root, stretch::geometry::Size{
            width: stretch::number::Number::Defined(viewport_data.width),
            height: stretch::number::Number::Defined(viewport_data.height),
        }).unwrap();

        let pos = mouse.get_mouse_position();
        let pos = Vec2::new(pos[0], pos[1]);

        let mut mouse_pressed = false;

        for event in input_events.read(&mut ui.reader_id) {
            match event {
                InputEvent::MouseMoved(new_position) => {
                    // check hover here
                }
                InputEvent::MouseButtonPressed(idx) => {
                    //println!("mouse button clicked: {}", idx);
                    if *idx == 0 {
                        mouse_pressed = true;
                    }
                }
                InputEvent::MouseButtonReleased(idx) => {
                    //println!("mouse button released: {:?}", idx);
                    // check button up here
                }
                InputEvent::KeyPressed(key_code) => {
                    //println!("key pressed");
                    // maybe check tab for focus?
                }
                _ => (),
            }
        }

        if mouse_pressed {
            for (entity_id, (ui_transform, _clickable)) in clickable_query.iter_entities_immutable(&world) {
                let entity_layout = ui.layout.layout(ui_transform.node).unwrap();
                let size = entity_layout.size;
                let location = entity_layout.location;
                let size = Vec2::new(size.width, size.height);
                let location = Vec2::new(location.x, location.y);

                let min = location;
                let max = location + size;
                //println!("Button positions: min: {:?}, max: {:?}", min, max);
                if pos.x > min.x && pos.y > min.y  && pos.x < max.x && pos.y < max.y {
                    ui_events.single_write(UiEvent{entity: entity_id, event_type: UiEventType::Clicked});

                    println!("button pressed: {}", entity_id);
                }

            }
        }
    }

    pub fn shut_down(world: &mut World) {
        world.resources.remove::<UiSystem>();
    }
}
