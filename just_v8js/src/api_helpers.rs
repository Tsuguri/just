use crate::EHM;

use just_core::ecs::prelude::World;

// pub fn add_function(guard: &ContextGuard, obj: &js::value::Object, name: &str, fun: Box<FunctionCallback>) {
//     let fun = js::value::Function::new(guard, fun);
//     obj.set(&guard, js::Property::new(&guard, name), fun);
// }

// pub fn world(ctx: &js::Context) -> &mut World {
//     *ctx.get_user_data_mut::<&mut World>().unwrap()
// }

// pub fn external_prototypes(ctx: &js::Context) -> &EHM {
//     *ctx.get_user_data::<&EHM>().unwrap()
// }
