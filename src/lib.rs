#![allow(clippy::type_complexity)]

mod audio;
mod core;
mod loading;
mod menu;
mod player;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;
use std::collections::HashMap;

use crate::core::camera::CameraPlugin;
use crate::core::state::GameState;
use bevy::app::App;
use bevy::asset::RenderAssetUsages;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use planetgen::PlanetData;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                CameraPlugin,
                LoadingPlugin,
                InternalAudioPlugin,
                MenuPlugin,
                PlayerPlugin,
            ))
            .add_systems(OnEnter(GameState::InGame), spawn_planet);

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

fn spawn_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let generator = planetgen::PlanetGenerator::new(20.0);
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
}
pub fn build_stitched_planet_mesh(planet: &PlanetData) -> Mesh {
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
                let (nx, ny, nz) = planetgen::cube_face_point(face_idx, u, v);
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
                    let color = planet.plates[plate_id].color;
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
