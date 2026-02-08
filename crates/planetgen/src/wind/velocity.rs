// Pure wind velocity calculation logic

use super::influence::MountainInfluenceMap;
use super::{DEFAULT_WIND_SPEED, SIGNS, TAU, TURN_POINTS, ZONAL_SIGNS};
use crate::config::WindDeflectionConfig;
use crate::planet::PlanetData;
use glam::Vec3;

/// Pure wind field calculations (no engine dependencies)
pub struct WindField;

impl WindField {
    /// Calculate wind velocity at a given position on the sphere
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    /// * `zonal_speed` - Speed of east/west movement
    ///
    /// # Returns
    /// Wind velocity vector tangent to the sphere surface
    pub fn calculate_wind_at(position: Vec3, zonal_speed: f32) -> Vec3 {
        let latitudinal_speed = Self::get_desired_latitudinal_speed(position);
        Self::get_velocity(position, latitudinal_speed, zonal_speed)
    }

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

        // Flip sign for southern hemisphere
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
        let z_sign = ZONAL_SIGNS[segment] + (ZONAL_SIGNS[segment + 1] - ZONAL_SIGNS[segment]) * s;

        // Get eastward direction
        let east_dir = Self::get_eastward_direction(position);

        // Return smoothly blended zonal velocity
        east_dir * (z_sign * zonal_speed)
    }

    /// Get northward direction for a position on the sphere
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Northward unit vector tangent to the sphere
    fn get_northward_direction(position: Vec3) -> Vec3 {
        let up = Vec3::Y;
        let east = up.cross(position).normalize();
        position.cross(east).normalize()
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

/// A single cube face storing pre-computed wind velocity vectors
#[derive(Clone)]
pub struct WindCubeFace {
    /// Grid of velocity vectors [y][x]
    pub velocities: Vec<Vec<Vec3>>,
}

/// Pre-computed wind velocity cube map for the entire planet
#[derive(Clone)]
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
                    let dir = cube_face_point(face_idx, u, v).normalize();

                    // Calculate wind velocity at this position
                    let velocity = WindField::calculate_wind_at(dir, zonal_speed);

                    faces[face_idx].velocities[y][x] = velocity;
                }
            }
        }

        Self { faces, resolution }
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

    /// Build a wind cube map with terrain-aware deflection.
    pub fn build_with_terrain(
        resolution: usize,
        zonal_speed: f32,
        planet: &PlanetData,
        config: &WindDeflectionConfig,
    ) -> (Self, MountainInfluenceMap) {
        let mut wind = Self::build(resolution, zonal_speed);
        let influence = MountainInfluenceMap::build(planet, resolution, config);
        wind.apply_deflection(&influence, config);
        (wind, influence)
    }

    /// Apply mountain deflection to wind velocities.
    fn apply_deflection(
        &mut self,
        influence: &MountainInfluenceMap,
        config: &WindDeflectionConfig,
    ) {
        for _ in 0..config.deflection_iterations {
            // Snapshot current velocities
            let snapshot: Vec<Vec<Vec<Vec3>>> =
                self.faces.iter().map(|f| f.velocities.clone()).collect();

            for face_idx in 0..6 {
                for y in 0..self.resolution {
                    let v = (y as f32 / (self.resolution - 1) as f32) * 2.0 - 1.0;
                    for x in 0..self.resolution {
                        let u = (x as f32 / (self.resolution - 1) as f32) * 2.0 - 1.0;

                        let dir = cube_face_point(face_idx, u, v).normalize();
                        let (cost, ridge_tangent) = influence.sample(dir);

                        if cost < 0.01 {
                            continue;
                        }

                        let wind = snapshot[face_idx][y][x];
                        let speed = wind.length();
                        if speed < 1e-6 {
                            continue;
                        }

                        let surface_normal = dir;

                        // Ridge normal = perpendicular to ridge tangent in tangent plane
                        let ridge_normal = surface_normal.cross(ridge_tangent);
                        let ridge_normal_len = ridge_normal.length();
                        if ridge_normal_len < 1e-6 {
                            continue;
                        }
                        let ridge_normal = ridge_normal / ridge_normal_len;

                        // Decompose wind
                        let v_along = ridge_tangent * wind.dot(ridge_tangent);
                        let across_component = wind.dot(ridge_normal);

                        // Redirect across-ridge energy along the ridge
                        // so wind flows around mountains, not through them
                        let along_sign = if wind.dot(ridge_tangent) >= 0.0 {
                            1.0
                        } else {
                            -1.0
                        };
                        let v_redirected = ridge_tangent * across_component.abs() * along_sign;

                        let deflected = v_along + v_redirected;

                        // Blend original and deflected by cost * strength
                        let blend = cost * config.deflection_strength;
                        let blended = wind.lerp(deflected, blend);

                        // Re-project to tangent plane
                        let tangent_v = blended - surface_normal * blended.dot(surface_normal);

                        // Restore original speed
                        let new_len = tangent_v.length();
                        let final_v = if new_len > 1e-6 {
                            tangent_v * (speed / new_len)
                        } else {
                            wind
                        };

                        self.faces[face_idx].velocities[y][x] = final_v;
                    }
                }
            }
        }
    }
}

/// Convert 2D cube face coordinates to 3D world coordinates
///
/// # Arguments
/// * `face_idx` - Cube face index (0-5)
/// * `u` - Horizontal coordinate in range [-1, 1]
/// * `v` - Vertical coordinate in range [-1, 1]
///
/// # Returns
/// 3D coordinates on unit cube surface
pub fn cube_face_point(face_idx: usize, u: f32, v: f32) -> Vec3 {
    match face_idx {
        0 => Vec3::new(1.0, v, -u),  // +X face
        1 => Vec3::new(-1.0, v, u),  // -X face
        2 => Vec3::new(u, 1.0, -v),  // +Y face
        3 => Vec3::new(u, -1.0, v),  // -Y face
        4 => Vec3::new(u, v, 1.0),   // +Z face
        5 => Vec3::new(-u, v, -1.0), // -Z face
        _ => Vec3::ZERO,
    }
}

/// Convert 3D direction to cube face coordinates
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
