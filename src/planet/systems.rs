use crate::helpers::mesh::arrow_mesh;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::{Color, LinearRgba};
use bevy::math::{Quat, Vec3};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use planetgen::generator::{PlanetGenerator, cube_face_point};
use planetgen::planet::PlanetData;
use std::collections::HashMap;

pub fn spawn_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let generator = PlanetGenerator::new(20.0);
    let planet_data = generator.generate();

    let mesh = build_stitched_planet_mesh(&planet_data);
    let mesh_handle = meshes.add(mesh);

    let material_handle = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 0.4),
        ..default()
    });

    commands.spawn((
        Mesh3d(mesh_handle),
        MeshMaterial3d(material_handle),
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    ));

    spawn_plate_direction_arrows(&mut commands, &mut meshes, &mut materials, &planet_data);
}

fn build_stitched_planet_mesh(planet: &PlanetData) -> Mesh {
    let size = planet.face_grid_size;
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut dir_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut vertex_indices = vec![vec![vec![0u32; size]; size]; 6];
    let mut next_index = 0u32;

    let quant_scale = (size - 1) as f32;

    for (face_idx, face) in planet.faces.iter().enumerate() {
        for y in 0..size {
            let v = (y as f32 / (size - 1) as f32) * 2.0 - 1.0;
            for x in 0..size {
                let u = (x as f32 / (size - 1) as f32) * 2.0 - 1.0;
                let (nx, ny, nz) = cube_face_point(face_idx, u, v);
                let dir = Vec3::new(nx, ny, nz).normalize();

                let key = (
                    (dir.x * quant_scale).round() as i32,
                    (dir.y * quant_scale).round() as i32,
                    (dir.z * quant_scale).round() as i32,
                );

                let idx = *dir_map.entry(key).or_insert_with(|| {
                    let height = face.heightmap[y][x];
                    let radius = planet.radius + height;
                    let pos = dir * radius;
                    positions.push([pos.x, pos.y, pos.z]);

                    let plate_id = planet.plate_map[face_idx][y][x];
                    let color = planet.plates[plate_id].debug_color;
                    colors.push(color);

                    let i = next_index;
                    next_index += 1;
                    i
                });

                vertex_indices[face_idx][y][x] = idx;
            }
        }
    }

    for face_idx in 0..6 {
        for y in 0..(size - 1) {
            for x in 0..(size - 1) {
                let i0 = vertex_indices[face_idx][y][x];
                let i1 = vertex_indices[face_idx][y][x + 1];
                let i2 = vertex_indices[face_idx][y + 1][x];
                let i3 = vertex_indices[face_idx][y + 1][x + 1];
                indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
            }
        }
    }

    let normals: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| Vec3::from(*p).normalize().to_array())
        .collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn spawn_plate_direction_arrows(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    planet: &PlanetData,
) {
    let arrow_mesh = arrow_mesh();
    let arrow_mesh_handle = meshes.add(arrow_mesh);
    let arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 0.4),
        emissive: LinearRgba::BLUE,
        ..default()
    });

    // Calculate the scale factor (10% of planet radius)
    let arrow_scale = planet.radius * 0.2;

    // For each plate, calculate its center position
    for (plate_idx, plate) in planet.plates.iter().enumerate() {
        // Calculate center position by finding the average position of all cells belonging to this plate
        let mut center = Vec3::ZERO;
        let mut count = 0;

        // Iterate through all faces and find cells belonging to this plate
        for (face_idx, face) in planet.faces.iter().enumerate() {
            for y in 0..planet.face_grid_size {
                for x in 0..planet.face_grid_size {
                    if planet.plate_map[face_idx][y][x] == plate_idx {
                        // Convert grid position to 3D position
                        let u = (x as f32 / (planet.face_grid_size - 1) as f32) * 2.0 - 1.0;
                        let v = (y as f32 / (planet.face_grid_size - 1) as f32) * 2.0 - 1.0;
                        let (nx, ny, nz) = cube_face_point(face_idx, u, v);
                        let dir = Vec3::new(nx, ny, nz).normalize();
                        let height = face.heightmap[y][x];
                        let pos = dir * (planet.radius + height);

                        center += pos;
                        count += 1;
                    }
                }
            }
        }

        // Calculate average position if we found any cells
        if count > 0 {
            center /= count as f32;
            // Normalize to the planet radius and add a small offset
            center = center.normalize() * (planet.radius + 1.0);

            // Get the movement direction of the plate
            let direction =
                Vec3::new(plate.direction.x, plate.direction.y, plate.direction.z).normalize();
            info!("Plate {} direction: {:?}", plate_idx, direction);

            // Get the surface normal at this position (pointing outward from center)
            let surface_normal = center.normalize();

            // Project the plate direction onto the tangent plane at this surface point
            // This removes the component of the direction that points toward/away from the center
            let tangent_direction =
                (direction - surface_normal * direction.dot(surface_normal)).normalize();

            // Calculate rotation to point in the tangent direction
            let default_direction = Vec3::Z;
            let rotation = Quat::from_rotation_arc(default_direction, tangent_direction);

            commands.spawn((
                Mesh3d(arrow_mesh_handle.clone()),
                MeshMaterial3d(arrow_material.clone()),
                Transform::from_translation(center)
                    .with_rotation(rotation)
                    .with_scale(Vec3::splat(arrow_scale)),
                GlobalTransform::default(),
            ));
        }
    }
}