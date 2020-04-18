use legion::prelude::*;

#[derive(Clone)]
pub struct GameObject {
    pub name: String,
    pub children: Vec<Entity>,
    pub parent: Option<Entity>,
}

#[derive(Clone)]
pub struct ObjectsToDelete(Vec<Entity>);

impl ObjectsToDelete {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl GameObject {
    pub fn new() -> Self {
        GameObject {
            name: "".to_string(),
            children: vec![],
            parent: Option::None,
        }
    }
    pub fn initialize(world: &mut World) {
        world.resources.insert(ObjectsToDelete::new());

    }
}

impl GameObject {
    pub fn get_name(world: &World, id: Entity) -> String {
        world.get_component::<GameObject>(id).unwrap().name.clone()
    }
    pub fn set_name(world: &mut World, id: Entity, new_name: String) {
        world.get_component_mut::<GameObject>(id).unwrap().name = new_name;
    }

    pub fn find_by_name(world: &World, name: &str) -> Vec<Entity> {
        Read::<GameObject>::query().iter_entities_immutable(world).filter(|(x, y)| {
            y.name == name
        }).map(|(x,_y)| x).collect()
    }

    pub fn delete(world: &mut World, id: Entity) {
        world.resources.get_mut::<ObjectsToDelete>().unwrap().0.push(id);
    }

    pub fn remove_marked(world: &mut World) {
        let mut to_destroy = world.resources.get_mut::<ObjectsToDelete>().unwrap();
        let objects = std::mem::replace(&mut to_destroy.0, vec![]);
        drop(to_destroy);
        for obj in objects.into_iter() {
            // might have been removed as child of other object
            if !world.is_alive(obj) {
                continue;
            }
            Self::remove_game_object(world, obj);
        }
    }

    pub fn remove_game_object(world: &mut World, id: Entity) {
        let data = (*world.get_component::<GameObject>(id).unwrap()).clone();
        for child in data.children {
            Self::remove_game_object(world, child);
        }
        Self::remove_single(world, id);
    }
    
    fn remove_single(world: &mut World, id: Entity) {
        super::TransformHierarchy::set_parent(world, id, None).unwrap();
        world.delete(id);
    }

    pub fn create_empty(world: &mut World) -> Entity {
        let go = GameObject::new();

        let ent_id = world.insert(
            (),
            vec![
                (super::transform::Transform::new(),go,),
            ],
        ).to_vec();
        ent_id[0]
    }
}
