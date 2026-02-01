// Wind direction and speed calculation module

use bevy::prelude::*;

/// Default resolution for wind cube map (per face)
pub const DEFAULT_CUBEMAP_RESOLUTION: usize = 64;

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
        if lat_deg < 0.0 { -v_des } else { v_des }
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
    /// Uses smoothstep blending at zone boundaries for gradual direction changes
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    /// * `zonal_speed` - Speed of east/west movement
    ///
    /// # Returns
    /// Desired zonal velocity vector (east/west tangent to sphere)
    fn get_desired_zonal_velocity(position: Vec3, zonal_speed: f32) -> Vec3 {
        // Get latitude in degrees
        let lat_rad = position.y.asin();
        let lat_deg = lat_rad.to_degrees();
        let abs_lat = lat_deg.abs();

        // Zonal direction signs at key latitudes:
        // 0°: -1 (east → west)
        // 30°: +1 (west → east)
        // 60°: -1 (east → west)
        // 90°: -1 (east → west)
        const ZONAL_SIGNS: [f32; 4] = [-1.0, 1.0, -1.0, -1.0];

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
        // This makes the transition gradual instead of instant
        let s = 3.0 * t * t - 2.0 * t * t * t;

        // Lerp between the signs at the segment endpoints
        // At boundaries, this will smoothly transition from -1 to +1 (or vice versa)
        // passing through 0 (stopped) in the middle
        let z_sign = ZONAL_SIGNS[segment] + (ZONAL_SIGNS[segment + 1] - ZONAL_SIGNS[segment]) * s;

        // Get eastward direction
        let east_dir = Self::get_eastward_direction(position);

        // Return smoothly blended zonal velocity
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
}

/// A single cube face storing pre-computed wind velocity vectors
#[derive(Clone)]
pub struct WindCubeFace {
    /// Grid of velocity vectors [y][x]
    pub velocities: Vec<Vec<Vec3>>,
}

/// Pre-computed wind velocity cube map for the entire planet
#[derive(Resource, Clone)]
pub struct WindCubeMap {
    /// Six cube faces storing wind velocities
    pub faces: [WindCubeFace; 6],
    /// Resolution of each face (grid size)
    pub resolution: usize,
}

impl WindCubeMap {
    /// Build a new wind cube map by pre-computing wind velocities
    ///
    /// # Arguments
    /// * `resolution` - Grid resolution per face (e.g., 64 means 64x64 grid per face)
    /// * `zonal_speed` - East/west wind speed parameter
    ///
    /// # Returns
    /// Pre-computed wind cube map ready for sampling
    pub fn build(resolution: usize, zonal_speed: f32) -> Self {
        info!("Building wind cube map with resolution {}x{} per face", resolution, resolution);

        let blank_face = WindCubeFace {
            velocities: vec![vec![Vec3::ZERO; resolution]; resolution],
        };

        let mut faces = [
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
        ];

        // Pre-compute wind velocity for each cell on each face
        for face_idx in 0..6 {
            for y in 0..resolution {
                let v = (y as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
                for x in 0..resolution {
                    let u = (x as f32 / (resolution - 1) as f32) * 2.0 - 1.0;

                    // Convert cube face coordinates to 3D direction
                    let dir = Vec3::from(cube_face_point(face_idx, u, v)).normalize();

                    // Calculate wind velocity at this position
                    let latitudinal_speed = WindField::get_desired_latitudinal_speed(dir);
                    let velocity = WindField::get_velocity(dir, latitudinal_speed, zonal_speed);

                    faces[face_idx].velocities[y][x] = velocity;
                }
            }
        }

        info!("Wind cube map built successfully");

        Self {
            faces,
            resolution,
        }
    }

    /// Sample wind velocity at a given position using bilinear interpolation
    ///
    /// # Arguments
    /// * `position` - Position on sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Interpolated wind velocity vector at this position
    pub fn sample(&self, position: Vec3) -> Vec3 {
        let dir = position.normalize();

        // Convert 3D direction to cube face coordinates
        let (face_idx, u, v) = direction_to_cube_uv(dir);

        // Convert u,v from [-1, 1] to grid coordinates [0, resolution-1]
        let fx = ((u + 1.0) * 0.5) * (self.resolution - 1) as f32;
        let fy = ((v + 1.0) * 0.5) * (self.resolution - 1) as f32;

        // Get integer grid cell indices
        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.resolution - 1);
        let y1 = (y0 + 1).min(self.resolution - 1);

        // Get fractional parts for interpolation
        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        // Bilinear interpolation
        let face = &self.faces[face_idx];
        let v00 = face.velocities[y0][x0];
        let v10 = face.velocities[y0][x1];
        let v01 = face.velocities[y1][x0];
        let v11 = face.velocities[y1][x1];

        let v0 = v00.lerp(v10, tx);
        let v1 = v01.lerp(v11, tx);
        v0.lerp(v1, ty)
    }
}

/// Convert 2D cube face coordinates to 3D world coordinates
///
/// Maps normalized coordinates (u, v) in range [-1, 1] on a specific cube face
/// to 3D coordinates on the unit cube surface.
///
/// # Arguments
/// * `face_idx` - Cube face index (0-5)
/// * `u` - Horizontal coordinate in range [-1, 1]
/// * `v` - Vertical coordinate in range [-1, 1]
///
/// # Returns
/// 3D coordinates (x, y, z) on unit cube surface
pub fn cube_face_point(face_idx: usize, u: f32, v: f32) -> (f32, f32, f32) {
    match face_idx {
        0 => (1.0, v, -u),   // +X face
        1 => (-1.0, v, u),   // -X face
        2 => (u, 1.0, -v),   // +Y face
        3 => (u, -1.0, v),   // -Y face
        4 => (u, v, 1.0),    // +Z face
        5 => (-u, v, -1.0),  // -Z face
        _ => (0.0, 0.0, 0.0),
    }
}

/// Convert 3D direction to cube face coordinates
///
/// Takes a normalized 3D direction vector and determines which cube face it's on,
/// along with the (u, v) coordinates within that face.
///
/// # Arguments
/// * `dir` - Normalized direction vector
///
/// # Returns
/// Tuple of (face_index, u, v) where u,v are in range [-1, 1]
pub fn direction_to_cube_uv(dir: Vec3) -> (usize, f32, f32) {
    let abs_x = dir.x.abs();
    let abs_y = dir.y.abs();
    let abs_z = dir.z.abs();

    // Find dominant axis to determine face
    if abs_x >= abs_y && abs_x >= abs_z {
        // X-axis dominant
        if dir.x > 0.0 {
            // +X face (0)
            let u = -dir.z / abs_x;
            let v = dir.y / abs_x;
            (0, u, v)
        } else {
            // -X face (1)
            let u = dir.z / abs_x;
            let v = dir.y / abs_x;
            (1, u, v)
        }
    } else if abs_y >= abs_x && abs_y >= abs_z {
        // Y-axis dominant
        if dir.y > 0.0 {
            // +Y face (2)
            let u = dir.x / abs_y;
            let v = -dir.z / abs_y;
            (2, u, v)
        } else {
            // -Y face (3)
            let u = dir.x / abs_y;
            let v = dir.z / abs_y;
            (3, u, v)
        }
    } else {
        // Z-axis dominant
        if dir.z > 0.0 {
            // +Z face (4)
            let u = dir.x / abs_z;
            let v = dir.y / abs_z;
            (4, u, v)
        } else {
            // -Z face (5)
            let u = -dir.x / abs_z;
            let v = dir.y / abs_z;
            (5, u, v)
        }
    }
}

