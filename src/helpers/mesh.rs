use bevy::asset::RenderAssetUsages;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};

pub fn arrow_mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    // Arrow shaft (unit length = 1.0, unit width = 0.4)
    positions.push([0.0, 0.0, 0.0]); // 0: base
    positions.push([0.0, 0.0, 0.7]); // 1: shaft tip (70% of unit length)

    // Arrow head
    positions.push([0.2, 0.0, 0.5]); // 2: right (width 0.4 / 2 = 0.2)
    positions.push([-0.2, 0.0, 0.5]); // 3: left
    positions.push([0.0, 0.2, 0.5]); // 4: top
    positions.push([0.0, -0.2, 0.5]); // 5: bottom

    // Shaft
    indices.extend_from_slice(&[0, 1, 2, 0, 2, 3, 0, 3, 1]);
    indices.extend_from_slice(&[0, 1, 4, 0, 4, 5, 0, 5, 1]);

    // Head
    indices.extend_from_slice(&[1, 2, 4, 1, 4, 3, 1, 3, 5, 1, 5, 2]);

    // Calculate simple normals
    for _ in 0..positions.len() {
        normals.push([0.0, 1.0, 0.0]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
