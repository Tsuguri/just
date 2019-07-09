use super::math::*;
use std::cell::RefCell;
use crate::scene::scripting::ScriptingEngine;


trait Ident {
    fn empty() -> Self;
}

impl Ident for Matrix {
    fn empty() -> Self {
        Matrix::identity()
    }
}

impl Ident for Quat {
    fn empty() -> Self {
        Quat::identity()
    }
}

struct ItemState<T: Ident> {
    pub changed: bool,
    pub item: T,
}
type MatrixState = ItemState<Matrix>;
type QuatState = ItemState<Quat>;

impl<T: Ident> ItemState<T> {
    fn new() -> Self {
        Self{
            changed: true,
            item: T::empty(),
        }
    }
}

pub struct GameObject {
    pub id: super::GameObjectId,
    pub children: Vec<super::GameObjectId>,
    pub parent: Option<super::GameObjectId>,

    pub position: RefCell<Vec3>,
    pub rotation: RefCell<Quat>,

    local_matrix: RefCell<MatrixState>,
    global_matrix: RefCell<MatrixState>,

}

impl GameObject {
    pub fn new(id: super::GameObjectId) -> Self {
        GameObject {
            id,
            children: vec![],
            parent: Option::None,

            position: Vec3::zeros().into(),
            rotation: Quat::identity().into(),

            local_matrix: RefCell::new(MatrixState::new()),
            global_matrix: RefCell::new(MatrixState::new()),
        }
    }
}
#[cfg(test)]
impl GameObject {
    pub fn valid_global(&self)->bool{
        !self.global_matrix.borrow().changed
    }

}

impl GameObject {
    pub fn void_local_matrix<E: ScriptingEngine>(&self, scene: &super::Scene<E>) {
        self.local_matrix.borrow_mut().changed=true;
        self.void_global_matrix(scene);

    }
    fn void_global_matrix<E: ScriptingEngine>(&self, scene: &super::Scene<E>) {
        if self.global_matrix.borrow().changed {
            return;
        }
        self.global_matrix.borrow_mut().changed=true;
        for child in &self.children {
            scene.object_data[*child].void_global_matrix(scene);
        }

    }

    pub fn get_local_matrix(&self) -> Matrix {
        let mut tr = self.local_matrix.borrow_mut();

        if tr.changed {
            tr.item = crate::glm::translation(&self.position.borrow()) * crate::glm::quat_to_mat4(&self.rotation.borrow());
            tr.changed = false;
        }
        tr.item
    }

    fn get_parent_matrix<E: ScriptingEngine>(&self, scene: &super::Scene<E>) -> Matrix {
        match self.parent {
            None => Matrix::identity(),
            Some(x) => scene.get_global_matrix(x),
        }
    }

    pub fn get_global_matrix<E: ScriptingEngine>(&self, scene: &super::Scene<E>) -> Matrix {
        let mut tr = self.global_matrix.borrow_mut();

        if tr.changed {
            let parent_matrix = self.get_parent_matrix(scene);
            tr.item = parent_matrix * self.get_local_matrix();
            tr.changed = false;
        }
        tr.item
    }

    pub fn get_global_position<E: ScriptingEngine>(&self, scene: &super::Scene<E>) -> Vec3 {
        let mat = self.get_parent_matrix(scene);
        pos(&(mat*pos_vec(&self.position.borrow())))
    }
    pub fn get_global_rotation<E: ScriptingEngine>(&self, scene: &super::Scene<E>) -> Quat {
        let parent_rotation = match self.parent {
            None => Quat::identity(),
            Some(x) => scene.get_global_rotation(x),
        };
        parent_rotation*(*self.rotation.borrow())
    }

    pub fn set_local_position<E: ScriptingEngine>(&self, scene: &super::Scene<E>, new_position: Vec3) {
        *self.position.borrow_mut() = new_position;
        self.void_local_matrix(scene);
    }

    pub fn get_local_position(&self) -> Vec3 {
        *self.position.borrow()
    }

    pub fn set_local_rotation<E: ScriptingEngine>(&self, scene: &super::Scene<E>, new_rotation: Quat) {
        *self.rotation.borrow_mut() = new_rotation;
        self.void_local_matrix(scene);
    }

    pub fn get_local_rotation(&self) -> Quat {
        *self.rotation.borrow()
    }
}