mod audio;
mod core;
mod helpers;
mod loading;
mod planet;
mod player;
mod ui;

use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::ui::UIPlugin;
use crate::planet::PlanetGenerationPlugin;

use crate::core::camera::CameraPlugin;
use crate::core::state::GameState;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use crate::planet::ui::menu::PlanetGenMenuPlugin;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                CameraPlugin,
                LoadingPlugin,
                InternalAudioPlugin,
                PlanetGenerationPlugin,
                PlanetGenMenuPlugin,
                UIPlugin,
            ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins(LogDiagnosticsPlugin::default());
        }
    }
}
