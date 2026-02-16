use crate::planet::components::ContinentViewMesh;
use crate::planet::precipitation::systems::PrecipitationCubeMap;
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::temperature::systems::TemperatureCubeMap;
use bevy::prelude::*;
use planetgen::biome::{BiomeColors, BiomeThresholds};

/// Tracks whether biome colors have been applied for the current planet.
/// Reset to false when a new planet is spawned or when biome settings change.
#[derive(Resource)]
pub struct BiomeColorState {
    pub applied: bool,
    // Snapshot of last-applied thresholds/colors to detect changes
    prev_ice_temp: f32,
    prev_tundra_temp: f32,
    prev_boreal_temp: f32,
    prev_temperate_temp: f32,
    prev_hot_temp: f32,
    prev_desert_precip: f32,
    prev_savanna_precip: f32,
    prev_jungle_precip: f32,
    prev_temperate_precip: f32,
    prev_ice_color: [f32; 3],
    prev_tundra_color: [f32; 3],
    prev_desert_color: [f32; 3],
    prev_savanna_color: [f32; 3],
    prev_temperate_color: [f32; 3],
    prev_jungle_color: [f32; 3],
    prev_land_temp_bonus: f32,
}

impl Default for BiomeColorState {
    fn default() -> Self {
        Self {
            applied: false,
            prev_ice_temp: f32::NAN,
            prev_tundra_temp: f32::NAN,
            prev_boreal_temp: f32::NAN,
            prev_temperate_temp: f32::NAN,
            prev_hot_temp: f32::NAN,
            prev_desert_precip: f32::NAN,
            prev_savanna_precip: f32::NAN,
            prev_jungle_precip: f32::NAN,
            prev_temperate_precip: f32::NAN,
            prev_ice_color: [f32::NAN; 3],
            prev_tundra_color: [f32::NAN; 3],
            prev_desert_color: [f32::NAN; 3],
            prev_savanna_color: [f32::NAN; 3],
            prev_temperate_color: [f32::NAN; 3],
            prev_jungle_color: [f32::NAN; 3],
            prev_land_temp_bonus: f32::NAN,
        }
    }
}

/// Build a BiomeColors struct from the current settings.
fn biome_colors_from_settings(settings: &PlanetGenerationSettings) -> BiomeColors {
    BiomeColors {
        ice: settings.biome_ice_color,
        tundra: settings.biome_tundra_color,
        desert: settings.biome_desert_color,
        savanna: settings.biome_savanna_color,
        temperate: settings.biome_temperate_color,
        jungle: settings.biome_jungle_color,
    }
}

/// Build a BiomeThresholds struct from the current settings.
fn biome_thresholds_from_settings(settings: &PlanetGenerationSettings) -> BiomeThresholds {
    BiomeThresholds {
        ice_temp: settings.biome_ice_temp,
        tundra_temp: settings.biome_tundra_temp,
        boreal_temp: settings.biome_boreal_temp,
        temperate_temp: settings.biome_temperate_temp,
        hot_temp: settings.biome_hot_temp,
        desert_precip: settings.biome_desert_precip,
        savanna_precip: settings.biome_savanna_precip,
        jungle_precip: settings.biome_jungle_precip,
        temperate_precip: settings.biome_temperate_precip,
    }
}

/// Updates continent mesh vertex colors with biome-based coloring
/// once both temperature and precipitation cubemaps are available.
pub fn update_continent_biome_colors(
    settings: Res<PlanetGenerationSettings>,
    temperature_cubemap: Option<Res<TemperatureCubeMap>>,
    precipitation_cubemap: Option<Res<PrecipitationCubeMap>>,
    mut biome_state: ResMut<BiomeColorState>,
    continent_query: Query<&Mesh3d, With<ContinentViewMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(temp_map) = temperature_cubemap else {
        return;
    };
    let Some(precip_map) = precipitation_cubemap else {
        return;
    };

    // Re-apply when underlying cubemaps are rebuilt (e.g. equator/pole temp change)
    if temp_map.is_changed() || precip_map.is_changed() {
        biome_state.applied = false;
    }

    // Detect if any biome-relevant setting changed since last apply
    let settings_changed =
        biome_state.prev_ice_temp != settings.biome_ice_temp
        || biome_state.prev_tundra_temp != settings.biome_tundra_temp
        || biome_state.prev_boreal_temp != settings.biome_boreal_temp
        || biome_state.prev_temperate_temp != settings.biome_temperate_temp
        || biome_state.prev_hot_temp != settings.biome_hot_temp
        || biome_state.prev_desert_precip != settings.biome_desert_precip
        || biome_state.prev_savanna_precip != settings.biome_savanna_precip
        || biome_state.prev_jungle_precip != settings.biome_jungle_precip
        || biome_state.prev_temperate_precip != settings.biome_temperate_precip
        || biome_state.prev_ice_color != settings.biome_ice_color
        || biome_state.prev_tundra_color != settings.biome_tundra_color
        || biome_state.prev_desert_color != settings.biome_desert_color
        || biome_state.prev_savanna_color != settings.biome_savanna_color
        || biome_state.prev_temperate_color != settings.biome_temperate_color
        || biome_state.prev_jungle_color != settings.biome_jungle_color
        || biome_state.prev_land_temp_bonus != settings.land_temperature_bonus;

    if settings_changed {
        biome_state.applied = false;
    }

    if biome_state.applied {
        return;
    }

    let planet_radius = settings.radius;
    let continent_threshold = settings.continent_threshold;
    let snow_threshold = settings.snow_threshold;
    let land_temp_bonus = settings.land_temperature_bonus;
    let biome_colors = biome_colors_from_settings(&settings);
    let biome_thresholds = biome_thresholds_from_settings(&settings);

    for mesh_handle in continent_query.iter() {
        let Some(mesh) = meshes.get_mut(&mesh_handle.0) else {
            continue;
        };

        let Some(positions_attr) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            continue;
        };
        let Some(positions) = positions_attr.as_float3() else {
            continue;
        };

        let positions_owned: Vec<[f32; 3]> = positions.to_vec();

        let colors = planetgen::mesh_data::calculate_biome_colors(
            &positions_owned,
            planet_radius,
            continent_threshold,
            snow_threshold,
            land_temp_bonus,
            &biome_colors,
            &biome_thresholds,
            |direction| temp_map.sample_temperature(direction),
            |direction| precip_map.sample(direction),
        );

        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }

    // Snapshot current settings
    biome_state.prev_ice_temp = settings.biome_ice_temp;
    biome_state.prev_tundra_temp = settings.biome_tundra_temp;
    biome_state.prev_boreal_temp = settings.biome_boreal_temp;
    biome_state.prev_temperate_temp = settings.biome_temperate_temp;
    biome_state.prev_hot_temp = settings.biome_hot_temp;
    biome_state.prev_desert_precip = settings.biome_desert_precip;
    biome_state.prev_savanna_precip = settings.biome_savanna_precip;
    biome_state.prev_jungle_precip = settings.biome_jungle_precip;
    biome_state.prev_temperate_precip = settings.biome_temperate_precip;
    biome_state.prev_ice_color = settings.biome_ice_color;
    biome_state.prev_tundra_color = settings.biome_tundra_color;
    biome_state.prev_desert_color = settings.biome_desert_color;
    biome_state.prev_savanna_color = settings.biome_savanna_color;
    biome_state.prev_temperate_color = settings.biome_temperate_color;
    biome_state.prev_jungle_color = settings.biome_jungle_color;
    biome_state.prev_land_temp_bonus = settings.land_temperature_bonus;

    biome_state.applied = true;
    info!("Biome colors applied to continent mesh");
}
