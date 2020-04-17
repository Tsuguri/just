use legion::prelude::Entity;

#[derive(Clone)]
pub struct GameObject {
    pub name: String,
    pub children: Vec<Entity>,
    pub parent: Option<Entity>,
}

impl GameObject {
    pub fn new() -> Self {
        GameObject {
            name: "".to_string(),
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
