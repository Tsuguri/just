use crate::math::*;
use std::cell::RefCell;

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

pub type MatrixState = ItemState<Matrix>;

impl<T: Ident> ItemState<T> {
    fn new() -> Self {
        Self {
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
