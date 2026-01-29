pub mod systems;

use bevy::prelude::*;

/// Number of particles to simulate
pub const PARTICLE_COUNT: u32 = 2500;

/// Resource to store wind particle settings
#[derive(Resource, Clone)]
pub struct WindParticleSettings {
    pub planet_radius: f32,
    pub particle_height_offset: f32,
    pub enabled: bool,
}

impl Default for WindParticleSettings {
    fn default() -> Self {
        Self {
            planet_radius: 50.0,
            particle_height_offset: 2.0,
            enabled: true,
        }
    }
}

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindParticleSettings>()
            .add_systems(Update, systems::update_wind_settings)
            .add_systems(Update, systems::handle_wind_tab_events)
            .add_systems(Update, systems::spawn_debug_particles);
    }
}

