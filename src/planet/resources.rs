use bevy::prelude::Resource;
use planetgen::constants::*;
use planetgen::planet::PlanetData;

#[derive(Resource, Clone)]
pub struct PlanetGenerationSettings {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
    pub show_arrows: bool,
    pub seed: u64,
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        Self {
            radius: (PLANET_MAX_RADIUS + PLANET_MIN_RADIUS) / 2.0,
            cells_per_unit: 2.0,
            num_plates: 15,
            num_micro_plates: 5,
            show_arrows: false,
            seed: 0,
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
