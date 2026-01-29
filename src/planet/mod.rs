pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod ui;
pub mod wind;
mod logic;

use crate::core::state::GameState;
use crate::planet::events::*;
use crate::planet::resources::*;
use crate::planet::systems::*;
use crate::planet::wind::WindPlugin;
use bevy::prelude::*;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(WindPlugin)
            .add_message::<GeneratePlanetEvent>()
            .add_message::<GenerateNewSeedEvent>()
            .add_message::<ToggleArrowsEvent>()
            .add_message::<SetCameraPositionEvent>()
            .add_message::<SettingsChanged>()
            .add_message::<WindTabActiveEvent>()
            .add_message::<PlanetSpawnedEvent>()
            .init_resource::<CurrentPlanetData>()
            .add_systems(
                OnEnter(GameState::PlanetGeneration),
                auto_generate_initial_planet,
            )
            .add_systems(Update, (spawn_planet_on_event, handle_arrow_toggle))
            .add_systems(
                Update,
                (
                    handle_camera_position_events,
                    handle_generate_new_seed,
                    planet_control,
                    smooth_camera_movement,
                )
                    .run_if(in_state(GameState::PlanetGeneration)),
            );
    }
}
