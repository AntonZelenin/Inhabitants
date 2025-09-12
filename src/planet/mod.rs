pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod ui;

use crate::core::state::GameState;
use crate::planet::events::*;
use crate::planet::resources::*;
use crate::planet::systems::*;
use bevy::prelude::*;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GeneratePlanetEvent>()
            .add_event::<GenerateNewSeedEvent>()
            .add_event::<ToggleArrowsEvent>()
            .add_event::<SetCameraPositionEvent>()
            .add_event::<SettingsChanged>()
            .init_resource::<CurrentPlanetData>()
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
