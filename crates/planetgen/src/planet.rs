use crate::generator::CubeFace;
use crate::plate::TectonicPlate;

pub enum PlateType {
    Continental,
    Oceanic,
}

pub enum PlateSizeClass {
    Regular,
    Micro,
}

pub struct PlanetData {
    pub faces: [CubeFace; 6],
    pub face_grid_size: usize,
    pub radius: f32,
    pub plate_map: Vec<Vec<Vec<usize>>>,
    pub plates: Vec<TectonicPlate>,
}
