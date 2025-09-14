use glam::Vec3;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};
use std::ops::Range;
use std::sync::{Mutex, OnceLock};

static CONFIG: OnceLock<Mutex<PlanetGenConfig>> = OnceLock::new();

/// Get a copy of the current configuration, loading from file if not already loaded
pub fn get_config() -> PlanetGenConfig {
    let config_mutex = CONFIG.get_or_init(|| {
        let config = PlanetGenConfig::load_from_file("planetgen_config.toml")
            .expect("Failed to load planetgen_config.toml - file must exist and be valid");
        Mutex::new(config)
    });
    config_mutex.lock().unwrap().clone()
}

pub fn reload_config() {
    reload_config_from_file("planetgen_config.toml").unwrap();
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetGenConfig {
    pub generation: GenerationConfig,
    pub plates: PlateConfig,
    pub boundaries: BoundaryConfig,
    pub flow_warp: FlowWarpConfig,
    pub microplates: MicroplateConfig,
    pub merging: MergingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub cells_per_unit: f32,
    pub continental_freq: f32,
    pub continental_amp: f32,
    pub oceanic_freq: f32,
    pub oceanic_amp: f32,
    pub planet_min_radius: f32,
    pub planet_max_radius: f32,
    pub default_num_plates: usize,
    pub default_num_micro_plates: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateConfig {
    pub min_separation_chord_distance: f32,
    pub continental_plate_probability: f64,
    pub micro_plate_weight_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryConfig {
    pub distortion_frequency: f32,
    pub distortion_amplitude: f32,
    pub warp_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowWarpConfig {
    pub default_freq: f32,
    pub default_amp: f32,
    pub default_steps: usize,
    pub default_step_angle: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroplateConfig {
    pub frequency_multiplier: f32,
    pub amplitude_multiplier: f32,
    pub jitter_range_min: f32,
    pub jitter_range_max: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergingConfig {
    pub selection_probability: f64,
    pub two_neighbors_probability: f64,
}

impl PlanetGenConfig {
    // todo can I simplify?
    pub fn default() -> Self {
        Self {
            generation: GenerationConfig {
                cells_per_unit: 5.0,
                continental_freq: 3.0,
                continental_amp: 0.7,
                oceanic_freq: 1.5, // CONTINENTAL_FREQ / 2.0
                oceanic_amp: 0.07, // CONTINENTAL_AMP / 10.0
                planet_min_radius: 30.0,
                planet_max_radius: 80.0,
                default_num_plates: 7,
                default_num_micro_plates: 6,
            },
            plates: PlateConfig {
                min_separation_chord_distance: 0.5,
                continental_plate_probability: 0.5,
                micro_plate_weight_factor: 2.7,
            },
            boundaries: BoundaryConfig {
                distortion_frequency: 7.0,
                distortion_amplitude: 0.2,
                warp_multiplier: 0.2,
            },
            flow_warp: FlowWarpConfig {
                default_freq: 0.8,
                default_amp: 0.25,
                default_steps: 2,
                default_step_angle: 0.1,
            },
            microplates: MicroplateConfig {
                frequency_multiplier: 1.5,
                amplitude_multiplier: 0.3,
                jitter_range_min: -0.1,
                jitter_range_max: 0.1,
            },
            merging: MergingConfig {
                selection_probability: 0.07,
                two_neighbors_probability: 0.2,
            },
        }
    }

    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: PlanetGenConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get the microplate jitter range as a Range<f32>
    pub fn microplate_jitter_range(&self) -> Range<f32> {
        self.microplates.jitter_range_min..self.microplates.jitter_range_max
    }
}

fn reload_config_from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let new_config = PlanetGenConfig::load_from_file(path)?;

    // Get the config mutex, creating it with the loaded config if it doesn't exist yet
    let config_mutex = CONFIG.get_or_init(|| {
        // This should rarely happen since get_config() is usually called first
        // But if reload is called before get_config, we'll initialize with the loaded config
        Mutex::new(new_config.clone())
    });

    // Update the existing config with the newly loaded one
    *config_mutex.lock().unwrap() = new_config;
    Ok(())
}
