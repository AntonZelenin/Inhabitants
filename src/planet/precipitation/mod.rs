pub mod systems;

use bevy::prelude::*;
use crate::planet::resources::PlanetGenerationSettings;

/// Resource to store precipitation visualization settings
#[derive(Resource, Clone)]
pub struct PrecipitationSettings {
    pub planet_radius: f32,
    pub enabled: bool,
    pub temperature_weight: f32,
    pub cubemap_resolution: usize,
}

impl Default for PrecipitationSettings {
    fn default() -> Self {
        let config = planetgen::get_config();
        Self {
            planet_radius: 50.0,
            enabled: false,
            temperature_weight: config.precipitation.temperature_weight,
            cubemap_resolution: config.precipitation.cubemap_resolution,
        }
    }
}

/// Resource that stores a copy of the last settings used to build the precipitation cubemap
#[derive(Resource, Clone)]
pub struct PreviousPrecipitationSettings(pub PlanetGenerationSettings);

impl Default for PreviousPrecipitationSettings {
    fn default() -> Self {
        Self(PlanetGenerationSettings::default())
    }
}

pub struct PrecipitationPlugin;

impl Plugin for PrecipitationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrecipitationSettings>()
            .init_resource::<PreviousPrecipitationSettings>()
            .add_systems(Startup, systems::initialize_precipitation_cubemap)
            .add_systems(Update, systems::update_precipitation_settings)
            .add_systems(Update, systems::regenerate_precipitation_meshes_on_settings_change)
            .add_systems(Update, systems::handle_precipitation_tab_events);
    }
}
