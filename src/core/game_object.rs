use crate::math::*;
use std::cell::RefCell;

use crate::traits::{GameObjectId, Controller};


pub trait Ident {
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ItemState<T: Ident> {
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

#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
    pub local_matrix: RefCell<MatrixState>,
    pub global_matrix: RefCell<MatrixState>,
}
unsafe impl Send for Transform {}
unsafe impl Sync for Transform {}

impl Transform {
    pub fn new() -> Self {
        Transform {
            position: Vec3::zeros(),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: Quat::identity(),
            local_matrix: RefCell::new(MatrixState::new()),
            global_matrix: RefCell::new(MatrixState::new()),
        }
    }
}

pub struct GameObject {
    pub name: String,
    pub id: GameObjectId,
    pub children: Vec<GameObjectId>,
    pub parent: Option<GameObjectId>,

    // pub position: RefCell<Vec3>,
    // pub rotation: RefCell<Quat>,
    // pub scale: RefCell<Vec3>,

    // local_matrix: RefCell<MatrixState>,
    // global_matrix: RefCell<MatrixState>,

}

impl GameObject {
    pub fn new(id: GameObjectId) -> Self {
        GameObject {
            name: "".to_string(),
            id,
            children: vec![],
            parent: Option::None,

            // position: Vec3::zeros().into(),
            // scale: Vec3::new(1.0, 1.0, 1.0).into(),
            // rotation: Quat::identity().into(),

            // local_matrix: RefCell::new(MatrixState::new()),
            // global_matrix: RefCell::new(MatrixState::new()),
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

    // pub fn get_global_matrix<C: Controller>(&self, world: &super::WorldData<C>) -> Matrix {
    //     let mut tr = self.global_matrix.borrow_mut();

    //     if tr.changed {
    //         let parent_matrix = self.get_parent_matrix(world);
    //         tr.item = parent_matrix * self.get_local_matrix();
    //         tr.changed = false;
    //     }
    //     tr.item
    // }
}