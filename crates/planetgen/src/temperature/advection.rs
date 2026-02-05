// Temperature advection by wind using semi-Lagrangian method

use glam::Vec3;
use super::data::{TemperatureCubeMap, TemperatureCubeFace, TemperatureField};
use crate::wind::WindCubeMap;

impl TemperatureCubeMap {
    /// Advect temperature by wind using semi-Lagrangian method
    /// 
    /// This pulls temperature values backward along wind trajectories,
    /// avoiding artifacts from forward (push) advection.
    /// 
    /// # Arguments
    /// * `wind` - Wind velocity cube map
    /// * `dt` - Time step (should be small relative to texel size)
    /// 
    /// # Returns
    /// New temperature cube map with advected values
    pub fn advect_by_wind(&self, wind: &WindCubeMap, dt: f32) -> Self {
        let blank_face = TemperatureCubeFace {
            temperatures: vec![vec![0.0; self.resolution]; self.resolution],
            colors: vec![vec![Vec3::ZERO; self.resolution]; self.resolution],
        };

        let mut new_faces = [
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
            blank_face.clone(),
        ];

        // Extract min/max from current temperature data to preserve color scale
        let (min_temp, max_temp) = self.find_temperature_range();

        // For each texel on each face
        for face_idx in 0..6 {
            for y in 0..self.resolution {
                let v = (y as f32 / (self.resolution - 1) as f32) * 2.0 - 1.0;
                
                for x in 0..self.resolution {
                    let u = (x as f32 / (self.resolution - 1) as f32) * 2.0 - 1.0;

                    // Current position on sphere (3D point)
                    let position = super::data::cube_face_point(face_idx, u, v).normalize();

                    // Get wind velocity at this position
                    let wind_velocity = wind.sample(position);

                    // Backtrace: move backward along wind
                    // p' = move_on_surface(p, -v * dt)
                    let backtraced_pos = move_on_sphere_surface(position, -wind_velocity * dt);

                    // Sample old temperature at backtraced position (bilinear, cross-face correct)
                    let temperature = self.sample_temperature(backtraced_pos);

                    // Compute color for this temperature
                    let color = TemperatureField::temperature_to_color(temperature, min_temp, max_temp);

                    // Store in new cubemap
                    new_faces[face_idx].temperatures[y][x] = temperature;
                    new_faces[face_idx].colors[y][x] = color;
                }
            }
        }

        Self {
            faces: new_faces,
            resolution: self.resolution,
        }
    }

    /// Find the temperature range in the current cubemap
    fn find_temperature_range(&self) -> (f32, f32) {
        let mut min_temp = f32::INFINITY;
        let mut max_temp = f32::NEG_INFINITY;

        for face in &self.faces {
            for row in &face.temperatures {
                for &temp in row {
                    min_temp = min_temp.min(temp);
                    max_temp = max_temp.max(temp);
                }
            }
        }

        (min_temp, max_temp)
    }
}

/// Move a point on the sphere surface along a tangent velocity vector
/// 
/// This ensures the point stays on the sphere surface during advection.
/// 
/// # Arguments
/// * `position` - Current position on sphere (normalized)
/// * `tangent_velocity` - Tangent velocity vector (already tangent to sphere)
/// 
/// # Returns
/// New position on sphere surface (normalized)
fn move_on_sphere_surface(position: Vec3, tangent_velocity: Vec3) -> Vec3 {
    // Simple Euler step: move along tangent
    let new_pos = position + tangent_velocity;
    
    // Project back onto sphere surface
    new_pos.normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_on_sphere_surface() {
        let pos = Vec3::new(1.0, 0.0, 0.0).normalize();
        let velocity = Vec3::new(0.0, 0.1, 0.0); // Tangent to sphere at (1,0,0)
        
        let new_pos = move_on_sphere_surface(pos, velocity);
        
        // Should still be on unit sphere
        assert!((new_pos.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_advection_preserves_resolution() {
        let temp_map = TemperatureCubeMap::build(16, 30.0, -20.0, -50.0, 50.0);
        let wind_map = WindCubeMap::build(16, 5.0);
        
        let advected = temp_map.advect_by_wind(&wind_map, 0.01);
        
        assert_eq!(advected.resolution, temp_map.resolution);
    }
}
