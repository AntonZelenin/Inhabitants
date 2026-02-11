// Precipitation probability map
//
// Step 1: Rising air → rain. Sinking air → dry.
// Uses the VerticalAirCubeMap to determine precipitation probability.
// Negative values (rising air / convergence) lead to higher precipitation.
// Positive values (sinking air / divergence) lead to lower precipitation.

use crate::temperature::TemperatureCubeMap;
use crate::wind::VerticalAirCubeMap;
use glam::Vec3;

/// Number of blur passes to create smooth precipitation zones
const BLUR_PASSES: usize = 15;

/// A single cube face storing precipitation probability values
#[derive(Clone)]
pub struct PrecipitationCubeFace {
    /// Grid of precipitation probability values [y][x], range [0.0, 1.0]
    pub values: Vec<Vec<f32>>,
}

/// Pre-computed precipitation probability cube map for the entire planet.
/// Currently based solely on vertical air movement (Step 1).
#[derive(Clone)]
pub struct PrecipitationCubeMap {
    pub faces: [PrecipitationCubeFace; 6],
    pub resolution: usize,
}

impl PrecipitationCubeMap {
    /// Build precipitation map from vertical air movement and temperature.
    ///
    /// Precipitation = uplift × moisture_capacity
    ///
    /// - Rising air (convergence) triggers precipitation
    /// - Temperature controls moisture capacity (Clausius-Clapeyron: ~7% more per °C)
    ///   - Warm air holds more moisture → higher precipitation potential
    ///   - Cold air holds less moisture → lower precipitation potential
    ///
    /// The `temperature_weight` controls how much temperature modulates the result.
    pub fn build(
        vertical_air: &VerticalAirCubeMap,
        temperature: Option<&TemperatureCubeMap>,
        temperature_weight: f32,
        equator_temp: f32,
        pole_temp: f32,
    ) -> Self {
        let resolution = vertical_air.resolution;
        let blank_face = PrecipitationCubeFace {
            values: vec![vec![0.0; resolution]; resolution],
        };

        let mut faces = [
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
        ];

        // Temperature range for normalization
        let temp_range = equator_temp - pole_temp;

        for face_idx in 0..6 {
            for y in 0..resolution {
                for x in 0..resolution {
                    let vertical = vertical_air.faces[face_idx].values[y][x];

                    // Uplift factor: rising air triggers precipitation
                    // -1 (rising) → 1.0, +1 (sinking) → 0.0
                    let uplift = (1.0 - vertical) / 2.0;

                    // Moisture capacity from temperature
                    // Warm = high capacity (1.0), Cold = low capacity (0.0)
                    let moisture_capacity = if let Some(temp_map) = temperature {
                        if temp_range.abs() > 0.01 {
                            let temp = temp_map.faces[face_idx].temperatures[y][x];
                            // Normalize: equator_temp → 1.0 (warm), pole_temp → 0.0 (cold)
                            ((temp - pole_temp) / temp_range).clamp(0.0, 1.0)
                        } else {
                            0.5
                        }
                    } else {
                        0.5
                    };

                    // Blend moisture capacity with weight
                    // At weight=0: capacity=1.0 (no temperature effect)
                    // At weight=1: capacity=moisture_capacity (full temperature effect)
                    let effective_capacity = 1.0 - temperature_weight * (1.0 - moisture_capacity);

                    // Precipitation = uplift × capacity
                    let precipitation = (uplift * effective_capacity).clamp(0.0, 1.0);
                    faces[face_idx].values[y][x] = precipitation;
                }
            }
        }

        // Apply blur passes to create smooth transitions between zones
        for _ in 0..BLUR_PASSES {
            for face_idx in 0..6 {
                faces[face_idx].values = blur_face(&faces[face_idx].values, resolution);
            }
        }

        Self { faces, resolution }
    }

    /// Sample precipitation probability at a given position using bilinear interpolation.
    ///
    /// Returns a value in [0.0, 1.0]: 0 = dry, 1 = maximum precipitation.
    pub fn sample(&self, position: Vec3) -> f32 {
        let dir = position.normalize();
        let (face_idx, u, v) = crate::wind::velocity::direction_to_cube_uv(dir);

        let fx = ((u + 1.0) * 0.5) * (self.resolution - 1) as f32;
        let fy = ((v + 1.0) * 0.5) * (self.resolution - 1) as f32;

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1).min(self.resolution - 1);
        let y1 = (y0 + 1).min(self.resolution - 1);

        let tx = fx - x0 as f32;
        let ty = fy - y0 as f32;

        let face = &self.faces[face_idx];
        let v00 = face.values[y0][x0];
        let v10 = face.values[y0][x1];
        let v01 = face.values[y1][x0];
        let v11 = face.values[y1][x1];

        let v0 = v00 + (v10 - v00) * tx;
        let v1 = v01 + (v11 - v01) * tx;
        v0 + (v1 - v0) * ty
    }
}

/// Apply a single box blur pass to a face grid.
fn blur_face(values: &[Vec<f32>], resolution: usize) -> Vec<Vec<f32>> {
    let mut out = vec![vec![0.0f32; resolution]; resolution];
    for y in 0..resolution {
        for x in 0..resolution {
            let mut sum = 0.0;
            let mut count = 0.0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && nx < resolution as i32 && ny >= 0 && ny < resolution as i32 {
                        sum += values[ny as usize][nx as usize];
                        count += 1.0;
                    }
                }
            }
            out[y][x] = sum / count;
        }
    }
    out
}

/// Convert precipitation probability to RGB color.
///
/// * 0.0 (dry): yellow
/// * 0.5 (moderate): light blue
/// * 1.0 (wet): blue
pub fn precipitation_to_color(value: f32) -> Vec3 {
    let t = value.clamp(0.0, 1.0);

    if t < 0.5 {
        // Dry to moderate: yellow → light blue
        let s = t * 2.0; // 0..1
        Vec3::new(
            1.0 - 0.5 * s,   // 1.0 → 0.5
            1.0 - 0.2 * s,   // 1.0 → 0.8
            0.2 + 0.8 * s,   // 0.2 → 1.0
        )
    } else {
        // Moderate to wet: light blue → blue
        let s = (t - 0.5) * 2.0; // 0..1
        Vec3::new(
            0.5 - 0.4 * s,   // 0.5 → 0.1
            0.8 - 0.4 * s,   // 0.8 → 0.4
            1.0,             // 1.0 → 1.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precipitation_color_range() {
        // Dry should be yellow
        let dry = precipitation_to_color(0.0);
        assert!(dry.x > 0.9); // red high
        assert!(dry.y > 0.9); // green high
        assert!(dry.z < 0.3); // blue low

        // Wet should be blue
        let wet = precipitation_to_color(1.0);
        assert!(wet.x < 0.2); // red low
        assert!(wet.z > 0.9); // blue high
    }
}
