pub mod systems;

use bevy::prelude::*;
use planetgen::temperature::DEFAULT_CUBEMAP_RESOLUTION;

/// Resource to store temperature visualization settings
#[derive(Resource, Clone)]
pub struct TemperatureSettings {
    pub planet_radius: f32,
    pub enabled: bool,
    pub temperature_cubemap_resolution: usize,
}

impl Default for TemperatureSettings {
    fn default() -> Self {
        Self {
            planet_radius: 50.0,
            enabled: false,
            temperature_cubemap_resolution: DEFAULT_CUBEMAP_RESOLUTION,
        }
    }
}

pub struct TemperaturePlugin;

impl Plugin for TemperaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TemperatureSettings>()
            .add_systems(Startup, systems::initialize_temperature_cubemap)
            .add_systems(Update, systems::update_temperature_settings)
            .add_systems(Update, systems::handle_temperature_tab_events)
            .add_systems(Update, systems::spawn_temperature_mesh);
    }
}
