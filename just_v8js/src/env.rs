use just_core::ecs::prelude::*;

use super::EHM;

pub struct JsEnvironment;

impl JsEnvironment {
    pub fn set_up(scope: &mut v8::HandleScope, world: &mut World, external_prototypes: &EHM) -> Self {
        insert_mut(scope, world);
        insert(scope, external_prototypes);
        Self {}
    }

    pub fn drop(self, scope: &mut v8::HandleScope) {
        scope.remove_slot::<&'static mut World>();
        scope.remove_slot::<&'static EHM>();

    }
}

fn insert<T: Send + Sync + 'static>(scope: &mut v8::HandleScope, val: &T) {
    let reference = unsafe { std::mem::transmute::<&T, &'static T>(val) };
    scope.set_slot(reference);
}

fn insert_mut<T: Send + Sync + 'static>(scope: &mut v8::HandleScope, val: &mut T) {
    let reference = unsafe { std::mem::transmute::<&mut T, &'static mut T>(val) };
    scope.set_slot(reference);
}
