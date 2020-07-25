use chakracore as js;
use legion::prelude::*;

use super::HM;

pub struct JsEnvironment;


impl JsEnvironment {
    pub fn set_up(context: &js::Context , world: &mut World, prototypes: &HM) -> Self {
        let reference = unsafe{
            std::mem::transmute::<&mut World, &'static mut World>(world)
        };
        context.insert_user_data::<&mut World>(reference);

        insert(context, prototypes);
        Self{}
    }

    pub fn drop(self, context: &js::Context) {
        debug_assert!(context.remove_user_data::<&mut World>().is_some());

        debug_assert!(context.remove_user_data::<&HM>().is_some());
    }
}

fn insert<T: Send + Sync + 'static>(context: &js::Context, val: &T){
    let reference = unsafe{
        std::mem::transmute::<&T, &'static T>(val)
    };
    context.insert_user_data::<&T>(reference);
}

fn insert_mut<T: Send + Sync + 'static>(context: &js::Context, val: &mut T){
        let reference = unsafe{
            std::mem::transmute::<&mut T, &'static mut T>(val)
        };
        context.insert_user_data::<&mut T>(reference);
    }
