use bevy::prelude::Resource;
use planetgen::constants::*;
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
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        let seed_8 = planetgen::tools::generate_seed8();
        Self {
            radius: (PLANET_MAX_RADIUS + PLANET_MIN_RADIUS) / 2.0,
            num_plates: DEFAULT_NUM_PLATES,
            num_micro_plates: DEFAULT_NUM_MICRO_PLATES,
            show_arrows: false,
            user_seed: seed_8,
            seed: planetgen::tools::expand_seed64(seed_8),
            flow_warp_freq: DEFAULT_FLOW_WARP_FREQ,
            flow_warp_steps: DEFAULT_FLOW_WARP_STEPS,
            flow_warp_step_angle: DEFAULT_FLOW_WARP_STEP_ANGLE,
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
