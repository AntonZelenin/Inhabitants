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

fn spawn_scene(mut commands: Commands, assets: Res<ModelAssets>) {
    commands.spawn(SceneRoot(assets.village.clone()));
}
