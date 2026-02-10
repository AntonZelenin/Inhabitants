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
    pub temperature_cubemap_resolution: usize,
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
            temperature_cubemap_resolution: config.temperature.cubemap_resolution,
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
