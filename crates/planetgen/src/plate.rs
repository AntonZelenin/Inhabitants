use crate::planet::PlateSizeClass;
use glam::Vec3;

pub struct TectonicPlate {
    pub id: usize,
    pub direction: Vec3,
    pub size_class: PlateSizeClass,
    pub debug_color: [f32; 4],
}
