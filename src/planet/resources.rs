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
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        let seed_8 = planetgen::tools::generate_seed8();
        Self {
            radius: (PLANET_MAX_RADIUS + PLANET_MIN_RADIUS) / 2.0,
            num_plates: 15,
            num_micro_plates: 5,
            show_arrows: false,
            user_seed: seed_8,
            seed: planetgen::tools::expand_seed64(seed_8),
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
