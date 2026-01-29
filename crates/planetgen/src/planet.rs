use crate::plate::TectonicPlate;
use crate::continents::ContinentNoiseConfig;
use crate::boundaries::BoundaryData;
use crate::wind::WindFace;

/// A single row on a cube face, containing plate IDs for each cell in that row
pub type FaceRow = Vec<usize>;
/// A complete grid for one cube face, containing multiple rows
pub type FaceGrid = Vec<FaceRow>;
/// The complete plate map for all 6 cube faces of the planet
pub type PlateMap = Vec<FaceGrid>;


pub enum PlateSizeClass {
    Regular,
    Micro,
}

#[derive(Clone)]
pub struct CubeFace {
    pub heightmap: Vec<Vec<f32>>,
}

pub struct PlanetData {
    pub faces: [CubeFace; 6],
    pub face_grid_size: usize,
    pub radius: f32,
    pub plate_map: PlateMap,
    pub plates: Vec<TectonicPlate>,
    /// Continent noise configuration for generating continents independently of plates
    pub continent_noise: ContinentNoiseConfig,
    /// Plate boundary interaction classifications (convergent/divergent/transform)
    pub boundary_data: BoundaryData,
    /// CPU-side wind field data (optional, generated on demand)
    pub wind_map: Option<[WindFace; 6]>,
}

impl PlanetData {
    /// Get a reference to the wind field, generating it if necessary
    pub fn get_wind_map(&mut self, wind_speed: f32) -> &[WindFace; 6] {
        self.ensure_wind_field(wind_speed);
        self.wind_map.as_ref().unwrap()
    }

    /// Generate or retrieve the wind field data
    /// If not already generated, creates a constant westward wind field
    pub fn ensure_wind_field(&mut self, wind_speed: f32) {
        if self.wind_map.is_none() {
            self.wind_map = Some(crate::wind::generate_constant_wind_field(
                self.face_grid_size,
                wind_speed,
            ));
        }
    }
}
