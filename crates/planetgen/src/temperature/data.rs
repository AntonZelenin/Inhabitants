// Pure temperature data calculation logic (engine-agnostic)

use super::{EQUATOR_TEMP, POLE_TEMP};
use glam::Vec3;

/// Pure temperature field calculations (no engine dependencies)
pub struct TemperatureField;

impl TemperatureField {
    /// Calculate temperature at a given position on the sphere based on latitude
    ///
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Temperature in Celsius
    pub fn calculate_temperature_at(position: Vec3) -> f32 {
        // Get latitude from Y component
        let lat_rad = position.y.asin();

        // Solar irradiance is proportional to cos(latitude)
        // This creates slow change near equator, dramatic change near poles
        // Physical explanation: sunlight hits equator perpendicularly (max energy per area),
        // but hits poles at shallow angle (same energy spread over larger area)
        let cos_lat = lat_rad.cos();

        // Map cos(lat) from [1.0 (equator) to 0.0 (pole)] to temperature range
        // cos_lat = 1.0 → EQUATOR_TEMP (35°C)
        // cos_lat = 0.0 → POLE_TEMP (-35°C)
        POLE_TEMP + (EQUATOR_TEMP - POLE_TEMP) * cos_lat
    }

    /// Convert temperature to a color for visualization
    ///
    /// # Arguments
    /// * `temp` - Temperature in Celsius
    ///
    /// # Returns
    /// RGB color as Vec3 (values in range 0.0-1.0)
    pub fn temperature_to_color(temp: f32) -> Vec3 {
        // Map temperature range [-35, 35] to [0, 1]
        let t = (temp - POLE_TEMP) / (EQUATOR_TEMP - POLE_TEMP);
        let t = t.clamp(0.0, 1.0);

        // Color gradient: light blue (cold) -> cyan -> green -> yellow -> orange -> red (hot)
        // Using multiple color stops for smooth transition
        
        if t < 0.2 {
            // Light blue to cyan (very cold: -35°C to -21°C)
            let local_t = t / 0.2;
            Vec3::new(
                0.5 + 0.0 * local_t,  // R: 0.5 -> 0.5
                0.8 + 0.2 * local_t,  // G: 0.8 -> 1.0
                1.0,                   // B: 1.0
            )
        } else if t < 0.4 {
            // Cyan to green (cold: -21°C to -7°C)
            let local_t = (t - 0.2) / 0.2;
            Vec3::new(
                0.5 - 0.3 * local_t,  // R: 0.5 -> 0.2
                1.0 - 0.2 * local_t,  // G: 1.0 -> 0.8
                1.0 - 0.5 * local_t,  // B: 1.0 -> 0.5
            )
        } else if t < 0.6 {
            // Green to yellow (mild: -7°C to 7°C)
            let local_t = (t - 0.4) / 0.2;
            Vec3::new(
                0.2 + 0.8 * local_t,  // R: 0.2 -> 1.0
                0.8 + 0.2 * local_t,  // G: 0.8 -> 1.0
                0.5 - 0.5 * local_t,  // B: 0.5 -> 0.0
            )
        } else if t < 0.8 {
            // Yellow to orange (warm: 7°C to 21°C)
            let local_t = (t - 0.6) / 0.2;
            Vec3::new(
                1.0,                   // R: 1.0
                1.0 - 0.5 * local_t,  // G: 1.0 -> 0.5
                0.0,                   // B: 0.0
            )
        } else {
            // Orange to red (hot: 21°C to 35°C)
            let local_t = (t - 0.8) / 0.2;
            Vec3::new(
                1.0,                   // R: 1.0
                0.5 - 0.5 * local_t,  // G: 0.5 -> 0.0
                0.0,                   // B: 0.0
            )
        }
    }
}

/// A single cube face storing pre-computed temperature values
#[derive(Clone)]
pub struct TemperatureCubeFace {
    /// Grid of temperature values in Celsius [y][x]
    pub temperatures: Vec<Vec<f32>>,
    /// Grid of color values [y][x]
    pub colors: Vec<Vec<Vec3>>,
}

/// Pre-computed temperature cube map for the entire planet
#[derive(Clone)]
pub struct TemperatureCubeMap {
    /// Six cube faces storing temperature data
    pub faces: [TemperatureCubeFace; 6],
    /// Resolution of each face (grid size)
    pub resolution: usize,
}

impl TemperatureCubeMap {
    /// Build a new temperature cube map by pre-computing temperatures
    ///
    /// # Arguments
    /// * `resolution` - Grid resolution per face (e.g., 64 means 64x64 grid per face)
    ///
    /// # Returns
    /// Pre-computed temperature cube map ready for sampling
    pub fn build(resolution: usize) -> Self {
        let blank_face = TemperatureCubeFace {
            temperatures: vec![vec![0.0; resolution]; resolution],
            colors: vec![vec![Vec3::ZERO; resolution]; resolution],
        };

        let mut faces = [
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
        ];

        // Pre-compute temperature for each cell on each face
        for face_idx in 0..6 {
            for y in 0..resolution {
                let v = (y as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
                for x in 0..resolution {
                    let u = (x as f32 / (resolution - 1) as f32) * 2.0 - 1.0;

                    // Convert cube face coordinates to 3D direction
                    let dir = cube_face_point(face_idx, u, v).normalize();

                    // Calculate temperature at this position
                    let temp = TemperatureField::calculate_temperature_at(dir);
                    let color = TemperatureField::temperature_to_color(temp);

                    faces[face_idx].temperatures[y][x] = temp;
                    faces[face_idx].colors[y][x] = color;
                }
            }
        }

        Self {
            faces,
            resolution,
        }
    }

    /// Sample temperature at a given position using bilinear interpolation
    ///
    /// # Arguments
    /// * `position` - Position on sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Interpolated temperature in Celsius at this position
    pub fn sample_temperature(&self, position: Vec3) -> f32 {
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
        let v00 = face.temperatures[y0][x0];
        let v10 = face.temperatures[y0][x1];
        let v01 = face.temperatures[y1][x0];
        let v11 = face.temperatures[y1][x1];

        let v0 = v00 + (v10 - v00) * tx;
        let v1 = v01 + (v11 - v01) * tx;
        v0 + (v1 - v0) * ty
    }

    /// Sample color at a given position using bilinear interpolation
    ///
    /// # Arguments
    /// * `position` - Position on sphere surface (normalized direction vector)
    ///
    /// # Returns
    /// Interpolated color as Vec3 at this position
    pub fn sample_color(&self, position: Vec3) -> Vec3 {
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
        let v00 = face.colors[y0][x0];
        let v10 = face.colors[y0][x1];
        let v01 = face.colors[y1][x0];
        let v11 = face.colors[y1][x1];

        let v0 = v00.lerp(v10, tx);
        let v1 = v01.lerp(v11, tx);
        v0.lerp(v1, ty)
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
        0 => Vec3::new(1.0, v, -u),   // +X face
        1 => Vec3::new(-1.0, v, u),   // -X face
        2 => Vec3::new(u, 1.0, -v),   // +Y face
        3 => Vec3::new(u, -1.0, v),   // -Y face
        4 => Vec3::new(u, v, 1.0),    // +Z face
        5 => Vec3::new(-u, v, -1.0),  // -Z face
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
