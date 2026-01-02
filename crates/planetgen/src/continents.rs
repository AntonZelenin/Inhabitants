use crate::config::NoiseConfig;
use glam::Vec3;

/// Multi-octave noise configuration for continent generation
pub struct ContinentNoiseConfig {
    /// Large-scale noise for continent placement (low frequency)
    pub continent_scale: NoiseConfig,
    /// Small-scale noise for coastline detail and local terrain variation
    pub detail_scale: NoiseConfig,
    /// Threshold for continent/ocean boundary (typically -0.3 to 0.3)
    pub continent_threshold: f32,
    /// Base elevation for ocean floor (negative value)
    pub ocean_floor_base: f32,
    /// Base elevation for continental crust (positive value)
    pub continent_base: f32,
}

impl ContinentNoiseConfig {
    /// Create a new continent noise configuration from config file
    pub fn new(seed_base: u32) -> Self {
        let config = crate::get_config();
        Self::from_config(seed_base, &config.continents)
    }

    /// Create from a ContinentConfig
    pub fn from_config(seed_base: u32, cfg: &crate::config::ContinentConfig) -> Self {
        Self {
            continent_scale: NoiseConfig::new(seed_base, cfg.continent_frequency, cfg.continent_amplitude),
            detail_scale: NoiseConfig::new(seed_base.wrapping_add(1), cfg.detail_frequency, cfg.detail_amplitude),
            continent_threshold: cfg.continent_threshold,
            ocean_floor_base: cfg.ocean_floor_base,
            continent_base: cfg.continent_base,
        }
    }

    /// Sample the multi-octave continent noise at a given position
    /// Returns the final height value, incorporating continent and detail layers
    /// (Mountains come from tectonic plate simulation)
    pub fn sample_height(&self, position: Vec3) -> f32 {
        // Sample both octaves
        let continent_value = self.continent_scale.sample(position);
        let detail_value = self.detail_scale.sample(position);

        // Add detail to the continent threshold for rough coastlines
        let adjusted_threshold = self.continent_threshold + (detail_value * 0.3);

        // Determine if this is continent or ocean
        if continent_value > adjusted_threshold {
            // Continental region: base elevation + detail variation
            let continent_factor = ((continent_value - self.continent_threshold)
                / (1.0 - self.continent_threshold))
                .clamp(0.0, 1.0);
            self.continent_base + (detail_value * 0.5 * continent_factor)
        } else {
            // Ocean region: base ocean floor + subtle detail
            self.ocean_floor_base + (detail_value * 0.2)
        }
    }

    /// Get just the continent mask (0.0 = ocean, 1.0 = continent)
    /// Useful for coloring or other effects
    pub fn sample_continent_mask(&self, position: Vec3) -> f32 {
        let continent_value = self.continent_scale.sample(position);
        let detail_value = self.detail_scale.sample(position);

        // Add detail to threshold for rough coastlines (same as in sample_height)
        let adjusted_threshold = self.continent_threshold + (detail_value * 0.3);

        if continent_value > adjusted_threshold {
            ((continent_value - self.continent_threshold) / (1.0 - self.continent_threshold))
                .clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

