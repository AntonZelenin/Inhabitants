use crate::plate::TectonicPlate;

/// A single row on a cube face, containing plate IDs for each cell in that row
pub type FaceRow = Vec<usize>;
/// A complete grid for one cube face, containing multiple rows
pub type FaceGrid = Vec<FaceRow>;
/// The complete plate map for all 6 cube faces of the planet
pub type PlateMap = Vec<FaceGrid>;

pub enum PlateType {
    Continental,
    // oceanic plates have lower amplitude and noise frequency, thus are smoother
    Oceanic,
}

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
}
