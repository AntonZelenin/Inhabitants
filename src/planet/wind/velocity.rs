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

    /// Get eastward direction for a position on the sphere
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Eastward unit vector tangent to the sphere (along lines of latitude)
    fn get_eastward_direction(position: Vec3) -> Vec3 {
        let world_north = Vec3::Y;
        let up = position.normalize();

        // Cross product: north × up = east
        let east_raw = world_north.cross(up);

        // Near poles, fallback to alternative calculation
        if east_raw.length_squared() < 1e-12 {
            let fallback = Vec3::X;
            fallback.cross(up).normalize()
        } else {
            east_raw.normalize()
        }
    }

    /// Get the desired zonal (east/west) velocity based on latitude
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    /// * `zonal_speed` - Speed of east/west movement
    ///
    /// # Returns
    /// Desired zonal velocity vector (east/west tangent to sphere)
    pub fn get_desired_zonal_velocity(position: Vec3, zonal_speed: f32) -> Vec3 {
        // Get latitude in degrees
        let lat_rad = position.y.asin();
        let lat_deg = lat_rad.to_degrees();
        let abs_lat = lat_deg.abs();

        // Determine zonal direction based on latitude band:
        // 0-30°: east → west (z_sign = -1)
        // 30-60°: west → east (z_sign = +1)
        // 60-90°: east → west (z_sign = -1)
        let z_sign = if abs_lat < 30.0 {
            -1.0 // east → west
        } else if abs_lat < 60.0 {
            1.0  // west → east
        } else {
            -1.0 // east → west
        };

        // Get eastward direction
        let east_dir = Self::get_eastward_direction(position);

        // Return zonal velocity
        east_dir * (z_sign * zonal_speed)
    }

    /// Get the wind velocity (meridional + zonal)
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    /// * `current_latitudinal_speed` - Current latitudinal velocity component
    /// * `zonal_speed` - Speed of east/west movement
    ///
    /// # Returns
    /// Velocity vector tangent to the sphere surface (north/south + east/west)
    pub fn get_velocity(position: Vec3, current_latitudinal_speed: f32, zonal_speed: f32) -> Vec3 {
        // Meridional (north/south) movement
        let north = Self::get_northward_direction(position);
        let meridional_velocity = north * current_latitudinal_speed;

        // Zonal (east/west) movement
        let zonal_velocity = Self::get_desired_zonal_velocity(position, zonal_speed);

        // Combine both components
        meridional_velocity + zonal_velocity
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
