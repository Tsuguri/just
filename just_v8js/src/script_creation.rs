use just_core::ecs::prelude::*;

pub struct ScriptCreationData {
    pub object: Entity,
    pub script_type: String,
}

pub struct ScriptCreationQueue {
    pub q: Vec<ScriptCreationData>,
}
