#![allow(clippy::type_complexity)]

mod audio;
mod core;
mod loading;
mod menu;
mod player;

use crate::audio::InternalAudioPlugin;
use crate::loading::{LoadingPlugin, ModelAssets};
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use crate::core::camera::CameraPlugin;
use crate::core::state::GameState;
use bevy::app::App;
use bevy::asset::RenderAssetUsages;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use planetgen::{CubeFace, PlanetData};

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

pub fn build_planet_mesh(planet: &PlanetData, base_radius: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let mut vertex_offset = 0;

    for (face_idx, face) in planet.faces.iter().enumerate() {
        let face_positions = generate_face_positions(face, face_idx, base_radius);

        let size = face.size;
        for y in 0..size {
            for x in 0..size {
                positions.push(face_positions[y * size + x]);
            }
        }

        for y in 0..(size - 1) {
            for x in 0..(size - 1) {
                let i = y * size + x;
                indices.extend_from_slice(&[
                    vertex_offset + i as u32,
                    vertex_offset + (i + size) as u32,
                    vertex_offset + (i + 1) as u32,
                    vertex_offset + (i + 1) as u32,
                    vertex_offset + (i + size) as u32,
                    vertex_offset + (i + size + 1) as u32,
                ]);
            }
        }

        vertex_offset += (size * size) as u32;
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn generate_face_positions(face: &CubeFace, face_idx: usize, base_radius: f32) -> Vec<[f32; 3]> {
    let mut positions = Vec::new();
    let size = face.size as f32;

    for y in 0..face.size {
        let v = (y as f32 / (size - 1.0)) * 2.0 - 1.0;
        for x in 0..face.size {
            let u = (x as f32 / (size - 1.0)) * 2.0 - 1.0;

            // cube face point
            let (nx, ny, nz) = match face_idx {
                0 => (1.0, v, -u),   // +X
                1 => (-1.0, v, u),   // -X
                2 => (u, 1.0, -v),   // +Y
                3 => (u, -1.0, v),   // -Y
                4 => (u, v, 1.0),   // +Z
                5 => (-u, v, -1.0),  // -Z
                _ => (0.0, 0.0, 0.0),
            };

            // NORMALIZE to project onto unit sphere
            let dir = Vec3::new(nx, ny, nz).normalize();

            // apply height displacement
            let height = face.heightmap[y][x];
            let r = base_radius + height;
            positions.push([dir.x * r, dir.y * r, dir.z * r]);
        }
    }

    positions
}

fn spawn_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let generator = planetgen::PlanetGenerator::new(100);
    let planet_data = generator.generate();

    let mesh = build_planet_mesh(&planet_data, 4.0);
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
