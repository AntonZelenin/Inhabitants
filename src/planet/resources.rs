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
    pub continent_threshold: f32,
    pub detail_frequency: f32,
    // View mode
    pub view_mode_plates: bool, // false = continents, true = plates
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        let config = planetgen::get_config();
        let seed_8 = planetgen::tools::generate_seed8();
        Self {
            radius: (config.generation.planet_max_radius + config.generation.planet_min_radius) / 2.0,
            num_plates: config.generation.default_num_plates,
            num_micro_plates: config.generation.default_num_micro_plates,
            show_arrows: false,
            user_seed: seed_8,
            seed: planetgen::tools::expand_seed64(seed_8),
            flow_warp_freq: config.flow_warp.default_freq,
            flow_warp_steps: config.flow_warp.default_steps,
            flow_warp_step_angle: config.flow_warp.default_step_angle,
            continent_frequency: config.continents.continent_frequency,
            continent_threshold: config.continents.continent_threshold,
            detail_frequency: config.continents.detail_frequency,
            view_mode_plates: false,
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
