use super::math::*;
use std::cell::RefCell;
use crate::scene::traits::*;


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

impl<T: Ident> ItemState<T> {
    fn new() -> Self {
        Self{
            changed: true,
            item: T::empty(),
        }
    }
}

pub struct GameObject {
    pub name: String,
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
            name: "".to_string(),
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
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, new_name: String) {
        self.name = new_name;
    }

    pub fn void_local_matrix<HW: Hardware>(&self, world: &super::WorldData<HW>) {
        self.local_matrix.borrow_mut().changed=true;
        self.void_global_matrix(world);

    }
    fn void_global_matrix<HW: Hardware>(&self, world: &super::WorldData<HW>) {
        if self.global_matrix.borrow().changed {
            return;
        }
        self.global_matrix.borrow_mut().changed=true;
        for child in &self.children {
            world.object_data[*child].void_global_matrix(world);
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

    fn get_parent_matrix<HW: Hardware>(&self, world: &super::WorldData<HW>) -> Matrix {
        match self.parent {
            None => Matrix::identity(),
            Some(x) => world.get_global_matrix(x),
        }
    }

    pub fn get_global_matrix<HW: Hardware>(&self, world: &super::WorldData<HW>) -> Matrix {
        let mut tr = self.global_matrix.borrow_mut();

        if tr.changed {
            let parent_matrix = self.get_parent_matrix(world);
            tr.item = parent_matrix * self.get_local_matrix();
            tr.changed = false;
        }
        tr.item
    }

    pub fn get_global_position<HW: Hardware>(&self, scene: &super::WorldData<HW>) -> Vec3 {
        let mat = self.get_parent_matrix(scene);
        pos(&(mat*pos_vec(&self.position.borrow())))
    }
    pub fn get_global_rotation<HW: Hardware>(&self, scene: &super::WorldData<HW>) -> Quat {
        let parent_rotation = match self.parent {
            None => Quat::identity(),
            Some(x) => scene.get_global_rotation(x),
        };
        parent_rotation*(*self.rotation.borrow())
    }

    pub fn set_local_position<HW: Hardware>(&self, scene: &super::WorldData<HW>, new_position: Vec3) {
        *self.position.borrow_mut() = new_position;
        self.void_local_matrix(scene);
    }

    pub fn get_local_position(&self) -> Vec3 {
        *self.position.borrow()
    }

    pub fn set_local_rotation<HW: Hardware>(&self, scene: &super::WorldData<HW>, new_rotation: Quat) {
        *self.rotation.borrow_mut() = new_rotation;
        self.void_local_matrix(scene);
    }

    pub fn get_local_rotation(&self) -> Quat {
        *self.rotation.borrow()
    }
}