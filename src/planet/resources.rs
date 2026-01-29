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
    pub ocean_wave_amplitude: f32,
    pub ocean_wave_frequency: f32,
    pub ocean_wave_speed: f32,
    pub ocean_normal_perturbation_scale: f32,
    // Wind visualization settings
    pub show_wind: bool,
    pub wind_particle_count: usize,
    pub wind_trail_length: f32,
    pub wind_particle_lifetime_min: f32,
    pub wind_particle_lifetime_max: f32,
    pub wind_particle_mesh_size: f32,
    pub wind_particle_height_offset: f32,
    pub wind_particle_stretch_multiplier: f32,
    pub wind_turn_rate: f32,
    pub wind_particle_trail_segments: usize,
    pub wind_particle_trail_width_ratio: f32,
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        let config = planetgen::get_config();
        let seed_8 = planetgen::tools::generate_seed8();
        Self {
            radius: (config.generation.planet_max_radius + config.generation.planet_min_radius)
                / 2.0,
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
            ocean_wave_amplitude: config.ocean.wave_amplitude,
            ocean_wave_frequency: config.ocean.wave_frequency,
            ocean_wave_speed: config.ocean.wave_speed,
            ocean_normal_perturbation_scale: config.ocean.normal_perturbation_scale,
            show_wind: false,
            wind_particle_count: config.wind.particle_count,
            wind_trail_length: config.wind.trail_length,
            wind_particle_lifetime_min: config.wind.particle_lifetime_min,
            wind_particle_lifetime_max: config.wind.particle_lifetime_max,
            wind_particle_mesh_size: config.wind.particle_mesh_size,
            wind_particle_height_offset: config.wind.particle_height_offset,
            wind_particle_stretch_multiplier: config.wind.particle_stretch_multiplier,
            wind_turn_rate: config.wind.turn_rate,
            wind_particle_trail_segments: config.wind.trail_segments,
            wind_particle_trail_width_ratio: config.wind.trail_width_ratio,
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
