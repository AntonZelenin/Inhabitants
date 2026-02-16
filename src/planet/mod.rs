pub mod biome;
pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod ui;
pub mod view;
pub mod wind;
pub mod temperature;
pub mod precipitation;
mod logic;

use crate::core::state::GameState;
use crate::planet::events::*;
use crate::planet::resources::*;
use crate::planet::systems::*;
use crate::planet::view::handle_tab_visibility;
use crate::planet::biome::BiomePlugin;
use crate::planet::wind::WindPlugin;
use crate::planet::temperature::TemperaturePlugin;
use crate::planet::precipitation::PrecipitationPlugin;
use bevy::prelude::*;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(BiomePlugin)
            .add_plugins(WindPlugin)
            .add_plugins(TemperaturePlugin)
            .add_plugins(PrecipitationPlugin)
            .add_message::<GeneratePlanetEvent>()
            .add_message::<GenerateNewSeedEvent>()
            .add_message::<ToggleArrowsEvent>()
            .add_message::<SetCameraPositionEvent>()
            .add_message::<SettingsChanged>()
            .add_message::<TabSwitchEvent>()
            .add_message::<WindTabActiveEvent>()
            .add_message::<TectonicTabActiveEvent>()
            .add_message::<TemperatureTabActiveEvent>()
            .add_message::<PrecipitationTabActiveEvent>()
            .add_message::<PlanetSpawnedEvent>()
            .add_message::<ResetCameraEvent>()
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
                    handle_reset_camera,
                    planet_control,
                    smooth_camera_movement,
                    handle_tab_visibility, // Centralized tab visibility handling
                )
                    .run_if(in_state(GameState::PlanetGeneration)),
            );
    }
}
