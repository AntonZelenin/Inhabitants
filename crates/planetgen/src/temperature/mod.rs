// Pure temperature simulation logic (engine-agnostic)

pub mod data;

pub use data::{TemperatureCubeFace, TemperatureCubeMap, TemperatureField};

/// Temperature constants
pub const EQUATOR_TEMP: f32 = 35.0; // Celsius at equator
pub const POLE_TEMP: f32 = -35.0; // Celsius at poles
pub const DEFAULT_CUBEMAP_RESOLUTION: usize = 64;
