pub mod systems;

use crate::core::state::GameState;
use bevy::prelude::*;
use systems::*;

pub struct TectonicPlugin;

impl Plugin for TectonicPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            handle_tab_visibility.run_if(in_state(GameState::PlanetGeneration)),
        );
    }
}
