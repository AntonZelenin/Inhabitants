pub mod components;
pub mod events;
pub mod resources;
pub mod systems;
pub mod ui;

use crate::planet::events::GeneratePlanetEvent;
use crate::planet::systems::spawn_planet_on_event;
use bevy::prelude::*;

pub struct PlanetGenerationPlugin;

impl Plugin for PlanetGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GeneratePlanetEvent>()
            .add_systems(Update, spawn_planet_on_event);
    }
}
