mod scene;

use slotmap;
use nalgebra_glm as glm;

fn main() {
    let _scene = scene::Scene::new();
    let mut mn = slotmap::SlotMap::new();
    let p1 = mn.insert(12);
    let p2 = mn.insert(13);
    let p3 = mn.insert(14);

    let mut sec = slotmap::SecondaryMap::new();
    sec.insert(p1, 22);

    for (k,v) in sec.iter() {
        println!("{:?}, {}", k, v)
    }
    mn.remove(p1);
    sec.remove(p1);
    for (k,v) in sec.iter() {
        println!("{:?}, {}", k, v)
    }
    println!("Hello, world!");
}
