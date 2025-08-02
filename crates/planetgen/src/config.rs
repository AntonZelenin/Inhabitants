use glam::Vec3;
use noise::{NoiseFn, Perlin};

pub struct NoiseConfig {
    perlin: Perlin,
    frequency: f32,
    amplitude: f32,
}

impl NoiseConfig {
    pub fn new(seed: u32, frequency: f32, amplitude: f32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            frequency,
            amplitude,
        }
    }

    pub fn sample(&self, dir: Vec3) -> f32 {
        let x = dir.x * self.frequency;
        let y = dir.y * self.frequency;
        let z = dir.z * self.frequency;
        self.perlin.get([x as f64, y as f64, z as f64]) as f32 * self.amplitude
    }
}
