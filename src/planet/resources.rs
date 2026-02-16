use bevy::prelude::Resource;
use planetgen::planet::PlanetData;

#[derive(Resource, Clone)]
pub struct PlanetGenerationSettings {
    pub radius: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
    pub show_arrows: bool,
    pub user_seed: u32,
    pub seed: u64,
    pub flow_warp_freq: f32,
    pub flow_warp_steps: usize,
    pub flow_warp_step_angle: f32,
    // Continent generation parameters
    pub continent_frequency: f32,
    pub continent_amplitude: f32,
    pub distortion_frequency: f32,
    pub distortion_amplitude: f32,
    pub continent_threshold: f32,
    pub detail_frequency: f32,
    pub detail_amplitude: f32,
    pub ocean_depth_amplitude: f32,
    // View mode
    pub view_mode_plates: bool, // false = continents, true = plates
    // Mountain snow threshold
    pub snow_threshold: f32,
    // Mountain generation
    pub mountain_height: f32,
    pub mountain_width: f32,
    // Ocean settings
    pub show_ocean: bool,
    // Wind visualization settings
    pub show_wind: bool,
    pub wind_particle_count: usize,
    pub wind_particle_height_offset: f32,
    pub wind_zonal_speed: f32,
    pub wind_particle_lifespan: f32,
    // Wind deflection settings
    pub wind_deflection_height_threshold: f32,
    pub wind_deflection_height_scale: f32,
    pub wind_deflection_spread_radius: usize,
    pub wind_deflection_spread_decay: f32,
    pub wind_deflection_strength: f32,
    pub wind_deflection_iterations: usize,
    // Vertical air movement
    pub show_vertical_air: bool,
    // Temperature visualization settings
    pub show_temperature: bool,
    pub land_temperature_bonus: f32, // Temperature increase for land (above sea level)
    pub temperature_equator_temp: f32,
    pub temperature_pole_temp: f32,
    pub temperature_max_temp: f32,
    pub temperature_min_temp: f32,
    pub temperature_latitude_falloff: f32,
    pub temperature_cubemap_resolution: usize,
    // Precipitation visualization settings
    pub show_precipitation: bool,
    pub precipitation_temperature_weight: f32,
    pub precipitation_ocean_weight: f32,
    pub precipitation_cubemap_resolution: usize,
    // Biome thresholds
    pub biome_ice_temp: f32,
    pub biome_tundra_temp: f32,
    pub biome_boreal_temp: f32,
    pub biome_temperate_temp: f32,
    pub biome_hot_temp: f32,
    pub biome_desert_precip: f32,
    pub biome_savanna_precip: f32,
    pub biome_jungle_precip: f32,
    pub biome_temperate_precip: f32,
    // Biome colors (RGB, 0.0-1.0)
    pub biome_ice_color: [f32; 3],
    pub biome_tundra_color: [f32; 3],
    pub biome_desert_color: [f32; 3],
    pub biome_savanna_color: [f32; 3],
    pub biome_temperate_color: [f32; 3],
    pub biome_jungle_color: [f32; 3],
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        let config = planetgen::get_config();
        let seed_8 = planetgen::tools::generate_seed8();
        Self {
            radius: config.generation.radius,
            num_plates: config.generation.default_num_plates,
            num_micro_plates: config.generation.default_num_micro_plates,
            show_arrows: false,
            user_seed: seed_8,
            seed: planetgen::tools::expand_seed64(seed_8),
            flow_warp_freq: config.flow_warp.default_freq,
            flow_warp_steps: config.flow_warp.default_steps,
            flow_warp_step_angle: config.flow_warp.default_step_angle,
            continent_frequency: config.continents.continent_frequency,
            continent_amplitude: config.continents.continent_amplitude,
            distortion_frequency: config.continents.distortion_frequency,
            distortion_amplitude: config.continents.distortion_amplitude,
            continent_threshold: config.continents.continent_threshold,
            detail_frequency: config.continents.detail_frequency,
            detail_amplitude: config.continents.detail_amplitude,
            ocean_depth_amplitude: config.continents.ocean_depth_amplitude,
            view_mode_plates: false,
            snow_threshold: config.mountains.snow_threshold,
            mountain_height: config.mountains.height,
            mountain_width: config.mountains.width,
            show_ocean: true,
            show_wind: false,
            wind_particle_count: config.wind.particle_count,
            wind_particle_height_offset: config.wind.particle_height_offset,
            wind_zonal_speed: config.wind.zonal_speed,
            wind_particle_lifespan: config.wind.particle_lifespan,
            wind_deflection_height_threshold: config.wind_deflection.height_threshold,
            wind_deflection_height_scale: config.wind_deflection.height_scale,
            wind_deflection_spread_radius: config.wind_deflection.spread_radius,
            wind_deflection_spread_decay: config.wind_deflection.spread_decay,
            wind_deflection_strength: config.wind_deflection.deflection_strength,
            wind_deflection_iterations: config.wind_deflection.deflection_iterations,
            show_vertical_air: false,
            show_temperature: false,
            land_temperature_bonus: config.temperature.land_temperature_bonus,
            temperature_equator_temp: config.temperature.equator_temp,
            temperature_pole_temp: config.temperature.pole_temp,
            temperature_max_temp: config.temperature.max_temp,
            temperature_min_temp: config.temperature.min_temp,
            temperature_latitude_falloff: config.temperature.latitude_falloff,
            temperature_cubemap_resolution: config.temperature.cubemap_resolution,
            show_precipitation: false,
            precipitation_temperature_weight: config.precipitation.temperature_weight,
            precipitation_ocean_weight: config.precipitation.ocean_weight,
            precipitation_cubemap_resolution: config.precipitation.cubemap_resolution,
            biome_ice_temp: config.biome.ice_temp,
            biome_tundra_temp: config.biome.tundra_temp,
            biome_boreal_temp: config.biome.boreal_temp,
            biome_temperate_temp: config.biome.temperate_temp,
            biome_hot_temp: config.biome.hot_temp,
            biome_desert_precip: config.biome.desert_precip,
            biome_savanna_precip: config.biome.savanna_precip,
            biome_jungle_precip: config.biome.jungle_precip,
            biome_temperate_precip: config.biome.temperate_precip,
            // DEBUG: vivid colors to identify biomes easily
            biome_ice_color: [1.0, 1.0, 1.0],       // white
            biome_tundra_color: [0.0, 0.0, 1.0],     // blue
            biome_desert_color: [1.0, 1.0, 0.0],     // yellow
            biome_savanna_color: [1.0, 0.5, 0.0],    // orange
            biome_temperate_color: [0.0, 1.0, 0.0],  // green
            biome_jungle_color: [1.0, 0.0, 0.0],     // red
        }
    }
}

#[derive(Resource)]
pub struct CurrentPlanetData {
    pub planet_data: Option<PlanetData>,
}

impl Default for CurrentPlanetData {
    fn default() -> Self {
        Self { planet_data: None }
    }
}
