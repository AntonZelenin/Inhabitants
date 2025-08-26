pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod ui;

use crate::planet::events::{GeneratePlanetEvent, ToggleArrowsEvent};
use crate::planet::systems::*;
use bevy::prelude::*;
use crate::planet::resources::CurrentPlanetData;

pub struct PlanetPlugin;

impl Plugin for PlanetPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GeneratePlanetEvent>()
            .add_event::<ToggleArrowsEvent>()
            .init_resource::<CurrentPlanetData>()
            .add_systems(Update, (spawn_planet_on_event, handle_arrow_toggle));
    }
}
