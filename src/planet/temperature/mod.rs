pub mod systems;

use bevy::prelude::*;
use crate::planet::resources::PlanetGenerationSettings;

/// Resource to store temperature visualization settings
#[derive(Resource, Clone)]
pub struct TemperatureSettings {
    pub planet_radius: f32,
    pub enabled: bool,
    pub temperature_cubemap_resolution: usize,
}

impl Default for TemperatureSettings {
    fn default() -> Self {
        let config = planetgen::get_config();
        Self {
            planet_radius: 50.0,
            enabled: false,
            temperature_cubemap_resolution: config.temperature.cubemap_resolution,
        }
    }
}

/// Resource that stores a copy of the last planet settings used to build the temperature cubemap
/// This allows us to detect when temperature values actually change
#[derive(Resource, Clone)]
pub struct PreviousPlanetSettings(pub PlanetGenerationSettings);

impl Default for PreviousPlanetSettings {
    fn default() -> Self {
        Self(PlanetGenerationSettings::default())
    }
}

pub struct TemperaturePlugin;

impl Plugin for TemperaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TemperatureSettings>()
            .init_resource::<PreviousPlanetSettings>()
            .add_systems(Startup, systems::initialize_temperature_cubemap)
            .add_systems(Update, systems::update_temperature_settings)
            .add_systems(Update, systems::regenerate_temperature_meshes_on_settings_change)
            .add_systems(Update, systems::handle_temperature_tab_events)
            .add_systems(Update, systems::advect_temperature_by_wind);
    }
}
