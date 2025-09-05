use bevy::prelude::Resource;
use planetgen::planet::PlanetData;
use planetgen::generator::PlanetSettings;

#[derive(Resource, Clone)]
pub struct PlanetGenerationSettings {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
    pub show_arrows: bool,
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        Self {
            radius: 20.0,
            cells_per_unit: 2.0,
            num_plates: 15,
            num_micro_plates: 5,
            show_arrows: true,
        }
    }
}

impl Into<PlanetSettings> for &PlanetGenerationSettings {
    fn into(self) -> PlanetSettings {
        PlanetSettings {
            radius: self.radius,
            cells_per_unit: self.cells_per_unit,
            num_plates: self.num_plates,
            num_micro_plates: self.num_micro_plates,
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