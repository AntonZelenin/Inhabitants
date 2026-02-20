// Precipitation probability map
//
// Step 1: Rising air → rain. Sinking air → dry.
// Uses the VerticalAirCubeMap to determine precipitation probability.
// Negative values (rising air / convergence) lead to higher precipitation.
// Positive values (sinking air / divergence) lead to lower precipitation.

use crate::planet::PlanetData;
use crate::temperature::TemperatureCubeMap;
use crate::wind::VerticalAirCubeMap;
use glam::Vec3;

/// Number of blur passes to create smooth precipitation zones
const BLUR_PASSES: usize = 5;

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
    /// Build precipitation map from vertical air movement, temperature, and terrain.
    ///
    /// Precipitation = uplift × moisture_capacity × water_availability
    ///
    /// - Rising air (convergence) triggers precipitation
    /// - Temperature controls moisture capacity (warm = high, cold = low)
    /// - Water availability: oceans evaporate more, land evaporates less
    ///   - Evaporation also scales with temperature (warm ocean = high evaporation)
    pub fn build(
        vertical_air: &VerticalAirCubeMap,
        temperature: Option<&TemperatureCubeMap>,
        planet: Option<&PlanetData>,
        temperature_weight: f32,
        ocean_weight: f32,
        equator_temp: f32,
        pole_temp: f32,
        continent_threshold: f32,
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
                    let normalized_temp = if let Some(temp_map) = temperature {
                        if temp_range.abs() > 0.01 {
                            let temp = temp_map.faces[face_idx].temperatures[y][x];
                            ((temp - pole_temp) / temp_range).clamp(0.0, 1.0)
                        } else {
                            0.5
                        }
                    } else {
                        0.5
                    };

                    // Moisture capacity: temperature only reduces precipitation in cold
                    // regions (cold air holds less moisture), but does NOT boost hot regions
                    // above what uplift provides. This prevents hot sinking-air zones
                    // (savannas) from getting unrealistic precipitation.
                    // Range: cold → (1 - temperature_weight), hot → 1.0
                    let moisture_cap = 1.0 - temperature_weight * (1.0 - normalized_temp);

                    // Water availability (evaporation source strength)
                    // Ocean = high evaporation, Land = low evaporation
                    let water_availability = if let Some(planet) = planet {
                        let u = (x as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
                        let v = (y as f32 / (resolution - 1) as f32) * 2.0 - 1.0;
                        let height = sample_heightmap(planet, face_idx, u, v);

                        let ocean_level = planet.radius + continent_threshold;
                        let is_ocean = height < ocean_level;

                        if is_ocean {
                            0.5 + 0.5 * normalized_temp
                        } else {
                            0.2 + 0.1 * normalized_temp
                        }
                    } else {
                        0.5
                    };

                    let effective_water = 1.0 - ocean_weight * (1.0 - water_availability);

                    // Precipitation is primarily driven by uplift. Moisture capacity
                    // and water availability act as limiters (via min), not boosters.
                    // This ensures sinking air always means low precipitation regardless
                    // of temperature.
                    let precipitation = (uplift * effective_water).min(moisture_cap).clamp(0.0, 1.0);
                    faces[face_idx].values[y][x] = precipitation;
                }
            }
        }

        // Apply blur passes with cross-face sampling to avoid edge seams
        let mut grids: [Vec<Vec<f32>>; 6] = std::array::from_fn(|i| faces[i].values.clone());
        for _ in 0..BLUR_PASSES {
            grids = crate::cubemap_utils::blur_cube_faces(&grids, resolution);
        }
        for (i, grid) in grids.into_iter().enumerate() {
            faces[i].values = grid;
        }

        // Normalize precipitation so the wettest point reaches 1.0
        let mut max_val = 0.0f32;
        for face in &faces {
            for row in &face.values {
                for &v in row {
                    max_val = max_val.max(v);
                }
            }
        }
        if max_val > 0.0 {
            let scale = 1.0 / max_val;
            for face in &mut faces {
                for row in &mut face.values {
                    for v in row {
                        *v *= scale;
                    }
                }
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

/// Sample heightmap at given cube face coordinates using bilinear interpolation.
fn sample_heightmap(planet: &PlanetData, face_idx: usize, u: f32, v: f32) -> f32 {
    let grid_size = planet.face_grid_size;
    let heightmap = &planet.faces[face_idx].heightmap;

    // Convert u,v from [-1, 1] to grid coordinates [0, grid_size-1]
    let fx = ((u + 1.0) * 0.5) * (grid_size - 1) as f32;
    let fy = ((v + 1.0) * 0.5) * (grid_size - 1) as f32;

    let x0 = (fx.floor() as usize).min(grid_size - 1);
    let y0 = (fy.floor() as usize).min(grid_size - 1);
    let x1 = (x0 + 1).min(grid_size - 1);
    let y1 = (y0 + 1).min(grid_size - 1);

    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;

    // Bilinear interpolation
    let h00 = heightmap[y0][x0];
    let h10 = heightmap[y0][x1];
    let h01 = heightmap[y1][x0];
    let h11 = heightmap[y1][x1];

    let h0 = h00 + (h10 - h00) * tx;
    let h1 = h01 + (h11 - h01) * tx;
    h0 + (h1 - h0) * ty
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
