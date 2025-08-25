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
use crate::planet::systems::spawn_planet;
use crate::ui::UIPlugin;

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
            .add_systems(OnEnter(GameState::MenuWithPlanet), setup_menu_with_planet)
            .add_systems(
                OnEnter(GameState::InGame),
                (spawn_planet, transition_to_menu_after_planet),
            )
            .add_systems(
                OnEnter(GameState::Loading),
                transition_to_menu_after_loading,
            );

        #[cfg(debug_assertions)]
        {
            app.add_plugins(LogDiagnosticsPlugin::default());
        }
    }
}

fn setup_menu_with_planet() {
    // This system will be handled by the MenuPlugin
}

fn transition_to_menu_after_loading(mut next_state: ResMut<NextState<GameState>>) {
    // Transition to MenuWithPlanet after loading
    next_state.set(GameState::MenuWithPlanet);
}

fn transition_to_menu_after_planet(mut next_state: ResMut<NextState<GameState>>) {
    // Immediately transition back to MenuWithPlanet after spawning planet
    next_state.set(GameState::MenuWithPlanet);
}