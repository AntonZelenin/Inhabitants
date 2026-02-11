// Precipitation probability map
//
// Step 1: Rising air → rain. Sinking air → dry.
// Uses the VerticalAirCubeMap to determine precipitation probability.
// Negative values (rising air / convergence) lead to higher precipitation.
// Positive values (sinking air / divergence) lead to lower precipitation.

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
    /// Build precipitation map from vertical air movement.
    ///
    /// Rising air (negative divergence) → higher precipitation probability
    /// Sinking air (positive divergence) → lower precipitation probability
    ///
    /// The `rising_air_weight` parameter controls how strongly rising air
    /// contributes to precipitation (0.0 to 1.0, default ~0.5).
    pub fn build_from_vertical_air(
        vertical_air: &VerticalAirCubeMap,
        rising_air_weight: f32,
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

        // Convert vertical air movement to precipitation probability
        // vertical_air values are in [-1, 1]: negative = rising, positive = sinking
        // We want: rising → high precipitation, sinking → low precipitation
        for face_idx in 0..6 {
            for y in 0..resolution {
                for x in 0..resolution {
                    let vertical = vertical_air.faces[face_idx].values[y][x];

                    // Convert: -1 (rising) → 1.0 (wet), +1 (sinking) → 0.0 (dry)
                    // Linear mapping: precipitation = (1 - vertical) / 2
                    // Then apply weight to control contribution strength
                    let base_precipitation = (1.0 - vertical) / 2.0;

                    // Apply weight (0.5 means this driver contributes 50% of final value)
                    let precipitation = base_precipitation * rising_air_weight;

                    faces[face_idx].values[y][x] = precipitation.clamp(0.0, 1.0);
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
/// * 0.0 (dry): yellow/tan
/// * 0.5 (moderate): green
/// * 1.0 (wet): blue
pub fn precipitation_to_color(value: f32) -> Vec3 {
    let t = value.clamp(0.0, 1.0);

    if t < 0.5 {
        // Dry to moderate: tan/yellow → green
        let s = t * 2.0; // 0..1
        Vec3::new(
            0.9 - 0.5 * s,   // 0.9 → 0.4
            0.8 - 0.1 * s,   // 0.8 → 0.7
            0.3 + 0.1 * s,   // 0.3 → 0.4
        )
    } else {
        // Moderate to wet: green → blue
        let s = (t - 0.5) * 2.0; // 0..1
        Vec3::new(
            0.4 - 0.3 * s,   // 0.4 → 0.1
            0.7 - 0.3 * s,   // 0.7 → 0.4
            0.4 + 0.5 * s,   // 0.4 → 0.9
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precipitation_color_range() {
        // Dry should be yellowish
        let dry = precipitation_to_color(0.0);
        assert!(dry.x > 0.8); // red high
        assert!(dry.y > 0.7); // green high
        assert!(dry.z < 0.5); // blue low

        // Wet should be bluish
        let wet = precipitation_to_color(1.0);
        assert!(wet.x < 0.3); // red low
        assert!(wet.z > 0.8); // blue high
    }
}
