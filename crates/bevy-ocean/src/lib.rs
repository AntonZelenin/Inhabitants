//! # Bevy Ocean Crate
//!
//! A stateless ocean renderer for Bevy 0.17.
//!
//! ## Usage
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_ocean::*;
//!
//! fn setup(
//!     mut commands: Commands,
//!     mut meshes: ResMut<Assets<Mesh>>,
//!     mut materials: ResMut<Assets<StandardMaterial>>,
//! ) {
//!     let config = OceanConfig {
//!         sea_level: 50.0, // planet_radius + continent_threshold
//!         grid_size: 64,
//!         wave_amplitude: 0.1,
//!         wave_frequency: 2.0,
//!         wave_speed: 1.0,
//!         ocean_color: Color::srgb(0.0, 0.4, 0.7),
//!     };
//!
//!     let ocean = OceanMeshBuilder::new(config)
//!         .with_time(0.0)
//!         .build();
//!
//!     commands.spawn((
//!         Mesh3d(meshes.add(ocean.mesh)),
//!         MeshMaterial3d(materials.add(ocean.material)),
//!     ));
//! }
//! ```

use bevy::asset::RenderAssetUsages;
use bevy::color::Color;
use bevy::math::Vec3;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::pbr::StandardMaterial;
use bevy::prelude::AlphaMode;

/// Configuration for ocean generation - your game provides this
#[derive(Debug, Clone, Copy)]
pub struct OceanConfig {
    /// Sea level - the radius at which the ocean sphere is rendered
    /// Typically: planet_radius + continent_threshold (relative to planet center)
    pub sea_level: f32,
    /// Number of grid subdivisions (higher = more detailed)
    pub grid_size: u32,
    /// Wave height amplitude
    pub wave_amplitude: f32,
    /// Wave frequency (how many waves per unit)
    pub wave_frequency: f32,
    /// Wave animation speed
    pub wave_speed: f32,
    /// Base ocean color
    pub ocean_color: Color,
}

impl Default for OceanConfig {
    fn default() -> Self {
        Self {
            sea_level: 50.0,
            grid_size: 64,
            wave_amplitude: 0.1,
            wave_frequency: 2.0,
            wave_speed: 1.0,
            ocean_color: Color::srgb(0.0, 0.4, 0.7),
        }
    }
}

/// Optional terrain height sampler - your game can provide this
/// Returns height at a given world position
pub type HeightSampler = Box<dyn Fn(Vec3) -> f32 + Send + Sync>;

/// The output of ocean generation - ready to render
pub struct OceanOutput {
    pub mesh: Mesh,
    pub material: StandardMaterial,
}

/// Builder for creating ocean meshes from external state
pub struct OceanMeshBuilder {
    config: OceanConfig,
    time: f32,
    height_sampler: Option<HeightSampler>,
}

impl OceanMeshBuilder {
    /// Create a new ocean mesh builder with the given configuration
    pub fn new(config: OceanConfig) -> Self {
        Self {
            config,
            time: 0.0,
            height_sampler: None,
        }
    }

    /// Set the current time for wave animation
    pub fn with_time(mut self, time: f32) -> Self {
        self.time = time;
        self
    }

    /// Set a terrain height sampler for wave interaction with terrain
    pub fn with_height_sampler(mut self, sampler: HeightSampler) -> Self {
        self.height_sampler = Some(sampler);
        self
    }

    /// Build the ocean mesh and material
    pub fn build(self) -> OceanOutput {
        let mesh = self.generate_mesh();
        let material = self.generate_material();

        OceanOutput { mesh, material }
    }

    fn generate_mesh(&self) -> Mesh {
        let size = self.config.grid_size;
        let radius = self.config.sea_level;

        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        // Generate spherical UV sphere with wave displacement
        for y in 0..=size {
            for x in 0..=size {
                let u = x as f32 / size as f32;
                let v = y as f32 / size as f32;

                // Spherical coordinates
                let theta = u * std::f32::consts::TAU; // longitude (0 to 2π)
                let phi = v * std::f32::consts::PI;    // latitude (0 to π)

                // Base sphere position at planet radius
                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let x_pos = radius * sin_phi * cos_theta;
                let y_pos = radius * cos_phi;
                let z_pos = radius * sin_phi * sin_theta;

                // No waves - perfect smooth sphere at sea_level
                let final_pos = Vec3::new(x_pos, y_pos, z_pos);

                positions.push([final_pos.x, final_pos.y, final_pos.z]);
                normals.push([sin_phi * cos_theta, cos_phi, sin_phi * sin_theta]);
                uvs.push([u, v]);
            }
        }

        // Generate indices for triangles
        for y in 0..size {
            for x in 0..size {
                let i0 = y * (size + 1) + x;
                let i1 = i0 + 1;
                let i2 = i0 + (size + 1);
                let i3 = i2 + 1;

                // Two triangles per quad
                indices.push(i0);
                indices.push(i2);
                indices.push(i1);

                indices.push(i1);
                indices.push(i2);
                indices.push(i3);
            }
        }

        // Normals are already correct for a perfect sphere, no need to recalculate

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    fn calculate_wave_height(&self, theta: f32, phi: f32) -> f32 {
        let freq = self.config.wave_frequency;
        let amp = self.config.wave_amplitude;
        let speed = self.config.wave_speed;
        let time = self.time;

        // Simple waves based on spherical coordinates
        let wave1 = (theta * freq + time * speed).sin() * amp;
        let wave2 = (phi * freq * 0.7 + time * speed * 0.8).sin() * amp * 0.5;
        let wave3 = ((theta + phi) * freq * 0.5 + time * speed * 1.2).sin() * amp * 0.3;

        wave1 + wave2 + wave3
    }

    fn calculate_normals(&self, positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
        let mut normals = vec![[0.0f32, 0.0, 0.0]; positions.len()];

        // Calculate face normals and accumulate
        for triangle in indices.chunks(3) {
            let i0 = triangle[0] as usize;
            let i1 = triangle[1] as usize;
            let i2 = triangle[2] as usize;

            let p0 = Vec3::from(positions[i0]);
            let p1 = Vec3::from(positions[i1]);
            let p2 = Vec3::from(positions[i2]);

            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            let normal = edge1.cross(edge2).normalize();

            // Accumulate normals for each vertex
            normals[i0][0] += normal.x;
            normals[i0][1] += normal.y;
            normals[i0][2] += normal.z;

            normals[i1][0] += normal.x;
            normals[i1][1] += normal.y;
            normals[i1][2] += normal.z;

            normals[i2][0] += normal.x;
            normals[i2][1] += normal.y;
            normals[i2][2] += normal.z;
        }

        // Normalize accumulated normals
        for normal in normals.iter_mut() {
            let n = Vec3::from(*normal).normalize();
            *normal = [n.x, n.y, n.z];
        }

        normals
    }

    fn generate_material(&self) -> StandardMaterial {
        StandardMaterial {
            base_color: self.config.ocean_color,
            metallic: 0.0,
            perceptual_roughness: 0.1,
            reflectance: 0.8,
            alpha_mode: AlphaMode::Blend, // Enable transparency
            unlit: false,
            double_sided: false,
            cull_mode: None,
            ..Default::default()
        }
    }
}

/// Query ocean wave height at a specific position on the sphere
/// Useful for physics/gameplay that needs to know the water surface
pub fn sample_ocean_height(config: &OceanConfig, position: Vec3, time: f32) -> f32 {
    // Convert position to spherical coordinates
    let radius = position.length();
    if radius < 0.001 {
        return config.sea_level;
    }

    let normalized = position / radius;
    let theta = normalized.z.atan2(normalized.x);
    let phi = normalized.y.acos();

    let freq = config.wave_frequency;
    let amp = config.wave_amplitude;
    let speed = config.wave_speed;

    let wave1 = (theta * freq + time * speed).sin() * amp;
    let wave2 = (phi * freq * 0.7 + time * speed * 0.8).sin() * amp * 0.5;
    let wave3 = ((theta + phi) * freq * 0.5 + time * speed * 1.2).sin() * amp * 0.3;

    config.sea_level + wave1 + wave2 + wave3
}

/// Sample ocean normal at a specific position
/// Useful for effects that need surface orientation
pub fn sample_ocean_normal(config: &OceanConfig, position: Vec3, time: f32) -> Vec3 {
    // For a sphere, approximate normal by sampling nearby points
    let epsilon = 0.01;

    let tangent1 = Vec3::new(-position.z, 0.0, position.x).normalize();
    let tangent2 = position.cross(tangent1).normalize();

    let h0 = sample_ocean_height(config, position, time);
    let h1 = sample_ocean_height(config, position + tangent1 * epsilon, time);
    let h2 = sample_ocean_height(config, position + tangent2 * epsilon, time);

    let d1 = h1 - h0;
    let d2 = h2 - h0;

    let base_normal = position.normalize();
    let perturbed = base_normal - tangent1 * (d1 / epsilon) - tangent2 * (d2 / epsilon);

    perturbed.normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocean_height_sampler() {
        let config = OceanConfig::default();
        let height = sample_ocean_height(&config, Vec3::new(1.0, 0.0, 0.0), 0.0);
        assert!(height >= config.sea_level - config.wave_amplitude * 2.0);
        assert!(height <= config.sea_level + config.wave_amplitude * 2.0);
    }

    #[test]
    fn test_ocean_normal_sampler() {
        let config = OceanConfig::default();
        let normal = sample_ocean_normal(&config, Vec3::new(1.0, 0.0, 0.0), 0.0);
        assert!((normal.length() - 1.0).abs() < 0.01, "Normal should be normalized");
    }

    #[test]
    fn test_ocean_builder() {
        let config = OceanConfig::default();
        let ocean = OceanMeshBuilder::new(config)
            .with_time(0.0)
            .build();

        // Just verify it doesn't panic and produces valid output
        assert!(ocean.mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_some());
        assert!(ocean.mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
    }
}
