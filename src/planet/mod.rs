pub mod systems;

use bevy::prelude::*;

/// Marker component for planet entities to enable despawning
#[derive(Component)]
pub struct PlanetEntity;
