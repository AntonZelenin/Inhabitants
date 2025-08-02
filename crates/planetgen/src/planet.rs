use crate::plate::TectonicPlate;

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
    pub plate_map: Vec<Vec<Vec<usize>>>,
    pub plates: Vec<TectonicPlate>,
}
