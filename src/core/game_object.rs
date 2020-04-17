use crate::traits::{GameObjectId, Controller};
use legion::prelude::Entity;

pub struct GameObject {
    pub name: String,
    pub id: GameObjectId,
    pub children: Vec<Entity>,
    pub parent: Option<Entity>,
}

impl GameObject {
    pub fn new(id: GameObjectId) -> Self {
        GameObject {
            name: "".to_string(),
            id,
            children: vec![],
            parent: Option::None,
        }
    }
}

impl GameObject {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }
}
