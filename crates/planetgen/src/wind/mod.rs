// Pure wind simulation logic

pub mod velocity;

pub use velocity::{WindCubeFace, WindCubeMap, WindField};

/// Wind constants
pub const DEFAULT_WIND_SPEED: f32 = 3.0;
pub const TAU: f32 = 0.8; // Smoothing time constant in seconds
pub const DEFAULT_CUBEMAP_RESOLUTION: usize = 64;

/// Turn points for wind circulation cells (in degrees latitude)
pub const TURN_POINTS: [f32; 4] = [0.0, 30.0, 60.0, 90.0];

/// Signs at each turn point in NORTHERN HEMISPHERE:
/// - towards the  equator = NEGATIVE (moving south)
/// - away from the  equator = POSITIVE (moving north)
///
/// 0° → towards the  equator = -1 (south)
/// 30° → away from the  equator = +1 (north)
/// 60° → towards the  equator = -1 (south)
/// 90° → towards the  equator = -1 (south)
pub const SIGNS: [f32; 4] = [-1.0, 1.0, -1.0, -1.0];

/// Zonal direction signs at key latitudes:
/// 0°: -1 (east → west)
/// 30°: +1 (west → east)
/// 60°: -1 (east → west)
/// 90°: -1 (east → west)
pub const ZONAL_SIGNS: [f32; 4] = [-1.0, 1.0, -1.0, -1.0];
