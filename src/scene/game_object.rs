
pub struct GameObject {
    pub id: super::GameObjectId,
    pub children: Vec<super::GameObjectId>,
    pub parent: Option<super::GameObjectId>,
}

impl GameObject {
    pub fn new(id: super::GameObjectId) -> Self {
        GameObject {
            id,
            children: vec![],
            parent: Option::None,
        }
    }
}