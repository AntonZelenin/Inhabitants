// Pure temperature simulation logic (engine-agnostic)

pub mod data;

pub use data::{TemperatureCubeFace, TemperatureCubeMap, TemperatureField};

/// Temperature constants
pub const EQUATOR_TEMP: f32 = 30.0; // Celsius at equator (generated range)
pub const POLE_TEMP: f32 = -30.0; // Celsius at poles (generated range)
pub const MAX_TEMP: f32 = 50.0; // Maximum possible temperature for color scale
pub const MIN_TEMP: f32 = -50.0; // Minimum possible temperature for color scale
pub const DEFAULT_CUBEMAP_RESOLUTION: usize = 64;
