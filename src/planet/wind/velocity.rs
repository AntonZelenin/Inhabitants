// Wind direction and speed calculation module

use bevy::prelude::*;

/// Wind constants
const DEFAULT_WIND_SPEED: f32 = 3.0;
const TURN_POINTS: [f32; 4] = [0.0, 30.0, 60.0, 90.0];
// Signs at each point in NORTHERN HEMISPHERE:
// towards equator = NEGATIVE (moving south), away from equator = POSITIVE (moving north)
// 0° → towards equator = -1 (south)
// 30° → away from equator = +1 (north)
// 60° → towards equator = -1 (south)
// 90° → towards equator = -1 (south)
// Between 0-30: goes from -3 to +3 (crosses 0 at ~15°)
// Between 30-60: goes from +3 to -3 (crosses 0 at ~45°)
// Between 60-90: stays at -3 (always towards equator)
const SIGNS: [f32; 4] = [-1.0, 1.0, -1.0, -1.0];
pub const TAU: f32 = 0.8; // Smoothing time constant in seconds

/// Calculate wind direction and speed at a given position on the sphere
pub struct WindField;

impl WindField {
    /// Get the desired latitudinal velocity based on position
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Desired latitudinal speed (scalar, positive = north, negative = south)
    pub fn get_desired_latitudinal_speed(position: Vec3) -> f32 {
        // Get latitude in degrees from Y component
        let lat_rad = position.y.asin();
        let lat_deg = lat_rad.to_degrees();

        // Work with absolute latitude for computation
        let abs_lat = lat_deg.abs();

        // Find which segment we're in
        let segment = if abs_lat < 30.0 {
            0
        } else if abs_lat < 60.0 {
            1
        } else {
            2
        };

        // Get segment endpoints
        let p0 = TURN_POINTS[segment];
        let p1 = TURN_POINTS[segment + 1];

        // Normalize position within segment [0, 1]
        let t = (abs_lat - p0) / (p1 - p0);

        // Smoothstep for smooth blending: s(t) = 3t² - 2t³
        let s = 3.0 * t * t - 2.0 * t * t * t;

        // Lerp between the signs at the segment endpoints
        let sign = SIGNS[segment] + (SIGNS[segment + 1] - SIGNS[segment]) * s;

        // Calculate desired latitudinal speed
        let v_des = DEFAULT_WIND_SPEED * sign;

        // Velocity convention:
        // - NEGATIVE velocity = moving south (decreasing latitude)
        // - POSITIVE velocity = moving north (increasing latitude)
        //
        // Northern Hemisphere (lat > 0):
        //   sign = -1 means "towards equator" = south = negative velocity ✓
        //   sign = +1 means "away from equator" = north = positive velocity ✓
        //
        // Southern Hemisphere (lat < 0):
        //   sign = -1 means "towards equator" = north = POSITIVE velocity (flip!)
        //   sign = +1 means "away from equator" = south = NEGATIVE velocity (flip!)
        //
        // So we flip the sign for southern hemisphere
        if lat_deg < 0.0 {
            -v_des
        } else {
            v_des
        }
    }

    /// Get northward direction for a position on the sphere
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Northward unit vector tangent to the sphere
    fn get_northward_direction(position: Vec3) -> Vec3 {
        // North direction is perpendicular to both the position (radial) and east
        // East is the cross product of Y-axis and position
        let up = Vec3::Y;
        let east = up.cross(position).normalize();
        // North is position × east
        position.cross(east).normalize()
    }

    /// Get the wind velocity (pure latitudinal movement)
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    /// * `current_latitudinal_speed` - Current latitudinal velocity component
    ///
    /// # Returns
    /// Velocity vector tangent to the sphere surface (north/south only)
    pub fn get_velocity(position: Vec3, current_latitudinal_speed: f32) -> Vec3 {
        // Pure latitudinal (north/south) movement only
        let north = Self::get_northward_direction(position);
        north * current_latitudinal_speed
    }

    /// Update latitudinal speed towards desired value using relaxation
    ///
    /// # Arguments
    /// * `current_speed` - Current latitudinal speed
    /// * `desired_speed` - Target latitudinal speed
    /// * `dt` - Delta time in seconds
    ///
    /// # Returns
    /// Updated latitudinal speed
    pub fn update_latitudinal_speed(current_speed: f32, desired_speed: f32, dt: f32) -> f32 {
        current_speed + (desired_speed - current_speed) * (dt / TAU)
    }
}
