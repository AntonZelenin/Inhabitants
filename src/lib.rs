mod audio;
mod core;
mod helpers;
mod loading;
mod menu;
mod planet;
mod player;
mod ui;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::ui::UIPlugin;
use crate::planet::systems::{despawn_planet, spawn_planet};

use crate::core::camera::CameraPlugin;
use crate::core::state::GameState;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;
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
                UIPlugin,
            ))
            .add_systems(OnEnter(GameState::InGame), (spawn_planet, transition_to_planet_menu))
            .add_systems(OnExit(GameState::InGame), despawn_planet)
            .add_systems(OnEnter(GameState::PlanetWithMenu), spawn_planet)
            .add_systems(OnExit(GameState::PlanetWithMenu), despawn_planet)
            .add_systems(OnEnter(GameState::Loading), || {
                // Transition to MainMenu after loading
            });

        #[cfg(debug_assertions)]
        {
            app.add_plugins(LogDiagnosticsPlugin::default());
        }
    }
}

fn transition_to_planet_menu(mut next_state: ResMut<NextState<GameState>>) {
    // Immediately transition back to PlanetWithMenu after spawning planet
    next_state.set(GameState::PlanetWithMenu);
}