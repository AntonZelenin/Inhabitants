use crate::config::NoiseConfig;
use crate::planet::PlateSizeClass;
use glam::Vec3;

pub struct TectonicPlate {
    pub id: usize,
    pub direction: Vec3,
    pub size_class: PlateSizeClass,
    pub noise_config: NoiseConfig,
    pub debug_color: [f32; 4],
}
