use bevy::prelude::*;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

/// Simplest possible material - just returns red
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct WindParticleMaterial {}

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
        Self {}
    }
}
