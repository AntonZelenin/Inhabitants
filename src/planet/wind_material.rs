use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Wind particle material with color uniform
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WindParticleMaterial {
    #[uniform(0)]
    pub color: LinearRgba,
}

impl Material for WindParticleMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/wind_particle.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

impl Default for WindParticleMaterial {
    fn default() -> Self {
        Self {
            color: LinearRgba::new(1.0, 1.0, 0.8, 1.0),
        }
    }
}
