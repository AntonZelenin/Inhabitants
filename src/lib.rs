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
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;

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
            .add_systems(OnEnter(GameState::InGame), spawn_scene);

        #[cfg(debug_assertions)]
        {
            app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

fn spawn_scene(
    mut commands: Commands,
    model_assets: Res<ModelAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn(SceneRoot(model_assets.village.clone()));
    commands.spawn((
        Transform::from_scale(Vec3::splat(10.0)),
        GlobalTransform::default(),
    )).with_children(|parent| {
        parent.spawn(SceneRoot(model_assets.village.clone()));
    });

    let ground_mesh = meshes.add(
        Plane3d::default()
            .mesh()
            .size(50.0, 50.0)
            .subdivisions(1),
    );

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb_u8(48, 64, 47),
            ..default()
        })),
    ));
}
