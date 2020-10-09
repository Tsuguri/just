use just_core::traits::scripting::{ScriptApiRegistry,function_params::Data};
use just_core::ecs::prelude::*;

struct TimeData {
    start: std::time::Instant,
    elapsed: f32,
    dt: f32,
}

pub struct TimeSystem;

impl TimeSystem {
    pub fn initialize(world: &mut World) {
        let system = TimeData {
            start: std::time::Instant::now(),
            elapsed: 0f32,
            dt: 0.016f32,
        };
        world.resources.insert(system);
    }

    pub fn update(world: &mut World) {
        let mut sys = <(Write<TimeData>)>::fetch(&world.resources);
        let duration = sys.start.elapsed();

        let elapsed = duration.as_secs() as f64 + duration.subsec_nanos() as f64 * 1e-9;
        let dt = elapsed - sys.elapsed as f64;
        sys.dt = dt as f32;
        sys.elapsed = elapsed as f32;
    }

    pub fn register_api<SAR: ScriptApiRegistry>(sar: &mut SAR) {
        let nm = sar.register_namespace("Time", None);

        sar.register_static_property(
            "elapsed",
            Some(&nm),
            Some(|d: Data<TimeData>| d.fetch.elapsed),
            Some(|()| {}),
        );

        //sar.register_function()
    }
}
