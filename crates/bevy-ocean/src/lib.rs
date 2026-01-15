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

        // Generate UV sphere - avoid seams by not duplicating vertices at poles/edges
        for y in 0..=size {
            for x in 0..=size {
                let u = x as f32 / size as f32;
                let v = y as f32 / size as f32;

                // Spherical coordinates
                let theta = u * std::f32::consts::TAU; // longitude (0 to 2π)
                let phi = v * std::f32::consts::PI;    // latitude (0 to π)

                let sin_phi = phi.sin();
                let cos_phi = phi.cos();
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let x_pos = radius * sin_phi * cos_theta;
                let y_pos = radius * cos_phi;
                let z_pos = radius * sin_phi * sin_theta;

                positions.push([x_pos, y_pos, z_pos]);
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

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }
}

impl OceanMeshBuilder {
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
