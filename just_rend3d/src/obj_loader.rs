use crate::{Mesh, RenderingManager};

pub(crate) fn load_obj_model(renderer: &RenderingManager, data: &[u8]) -> Mesh {
    let mut cursor = std::io::Cursor::new(data);
    let (mut objects, materials) =
        tobj::load_obj_buf(&mut cursor, true, |arg| Result::Err(tobj::LoadError::ReadError)).unwrap();

    let first_mesh = objects.pop().unwrap();
    println!("processing mesh {}", first_mesh.name);
    let mut first_mesh = first_mesh.mesh;

    let num_vertices = first_mesh.positions.len() / 3;
    let mut positions = Vec::with_capacity(num_vertices);
    let mut normals = Vec::with_capacity(num_vertices);
    let mut uvs = Vec::with_capacity(num_vertices);

    if first_mesh.normals.is_empty() {
        first_mesh.normals.resize(num_vertices * 3, 0.5);
        println!(
            "setting normals with {}, results in {} normals",
            num_vertices * 3,
            first_mesh.normals.len()
        );
    }
    if first_mesh.texcoords.is_empty() {
        first_mesh.texcoords.resize(num_vertices * 2, 0.5);
        println!(
            "setting uvs with {}, results in {} uvs",
            num_vertices * 2,
            first_mesh.texcoords.len()
        );
    }
    for i in 0..first_mesh.positions.len() / 3 {
        positions.push(glam::Vec3::new(
            first_mesh.positions[i * 3],
            first_mesh.positions[i * 3 + 1],
            first_mesh.positions[i * 3 + 2],
        ));
        normals.push(glam::Vec3::new(
            first_mesh.normals[i * 3],
            first_mesh.normals[i * 3 + 1],
            first_mesh.normals[i * 3 + 2],
        ));
        uvs.push(glam::Vec2::new(
            first_mesh.texcoords[i * 2],
            first_mesh.texcoords[i * 2 + 1],
        ));
    }

    let mesh = rend3::types::MeshBuilder::new(positions, rend3::types::Handedness::Right)
        .with_vertex_normals(normals)
        .with_vertex_uv0(uvs)
        .with_indices(first_mesh.indices)
        .build()
        .unwrap();
    Mesh {
        handle: renderer.renderer.add_mesh(mesh),
    }
}
