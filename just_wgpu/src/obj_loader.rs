use core::num;

use wgpu::util::DeviceExt;

use crate::{
    model::{MeshData, MeshVertex},
    Mesh, RenderingManager,
};

pub(crate) fn load_obj_model(renderer: &mut RenderingManager, data: &[u8], name: &str) -> Mesh {
    let mut cursor = std::io::Cursor::new(data);
    let (mut objects, materials) = tobj::load_obj_buf(
        &mut cursor,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |arg| Result::Err(tobj::LoadError::ReadError),
    )
    .unwrap();

    let first_mesh = objects.pop().unwrap();
    println!("processing mesh {}", first_mesh.name);
    let mut first_mesh = first_mesh.mesh;

    let num_vertices = first_mesh.positions.len() / 3;

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

    let buff = (0..first_mesh.positions.len() / 3)
        .map(|i| MeshVertex {
            position: [
                first_mesh.positions[i * 3],
                first_mesh.positions[i * 3 + 1],
                first_mesh.positions[i * 3 + 2],
            ],
            tex_coords: [first_mesh.texcoords[i * 2], first_mesh.texcoords[i * 2 + 1]],
            normal: [
                first_mesh.normals[i * 3],
                first_mesh.normals[i * 3 + 1],
                first_mesh.normals[i * 3 + 2],
            ],
        })
        .collect::<Vec<_>>();

    let vertex_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Vertex Buffer", name)),
        contents: bytemuck::cast_slice(&buff),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = renderer.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{} Index Buffer", name)),
        contents: bytemuck::cast_slice(&first_mesh.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let last_key = renderer.meshes.keys().map(|i| i.0).max().unwrap_or(0);
    let new_key = last_key + 1;
    renderer.meshes.insert(
        Mesh(new_key),
        MeshData {
            vertex_buffer,
            index_buffer,
            num_elements: first_mesh.indices.len() as u32,
        },
    );

    Mesh(new_key)
}
