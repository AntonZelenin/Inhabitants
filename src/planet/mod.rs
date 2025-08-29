pub mod components;
pub mod constants;
pub mod events;
pub mod resources;
pub mod systems;
pub mod ui;

use crate::core::state::GameState;
use crate::planet::events::{GeneratePlanetEvent, SetCameraPositionEvent, ToggleArrowsEvent};
use crate::planet::resources::CurrentPlanetData;
use crate::planet::systems::*;
use bevy::prelude::*;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GeneratePlanetEvent>()
            .add_event::<ToggleArrowsEvent>()
            .add_event::<SetCameraPositionEvent>()
            .init_resource::<CurrentPlanetData>()
            .add_systems(Update, (spawn_planet_on_event, handle_arrow_toggle))
            .add_systems(
                Update,
                (planet_control, smooth_camera_movement, handle_camera_position_events)
                    .run_if(in_state(GameState::PlanetGeneration)),
            );
    }
}