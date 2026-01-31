pub mod velocity;
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
    pub zonal_speed: f32,
    pub particle_lifespan: f32,
    pub density_bin_deg: f32,
    pub density_pressure_strength: f32,
    pub uplift_zone_deg: f32,
    pub fade_in_duration: f32,
    pub fade_out_duration: f32,
    pub source_zone_min_deg: f32,
    pub source_zone_max_deg: f32,
}

impl Default for WindParticleSettings {
    fn default() -> Self {
        Self {
            planet_radius: 50.0,
            particle_height_offset: 2.0,
            enabled: true,
            zonal_speed: 5.0,
            particle_lifespan: 4.0,
            density_bin_deg: 2.0,
            density_pressure_strength: 5.0,
            uplift_zone_deg: 5.0,
            fade_in_duration: 0.6,
            fade_out_duration: 0.6,
            source_zone_min_deg: 25.0,
            source_zone_max_deg: 35.0,
        }
    }
}

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WindParticleSettings>()
            .add_systems(Update, systems::update_wind_settings)
            .add_systems(Update, systems::handle_wind_tab_events)
            .add_systems(Update, systems::spawn_debug_particles)
            .add_systems(Update, systems::update_particles)
            .add_systems(Update, systems::update_particle_fade);
    }
}
