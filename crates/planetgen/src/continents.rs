//! # Continent Generation System
//!
//! This module provides two systems for generating planetary continents:
//!
//! ## Simple System (`ContinentNoiseConfig`)
//! - Two-layer noise: base continent shape + surface detail
//! - Fast and straightforward
//! - Good for testing and simple planets
//!
//! ## Advanced System (`AdvancedContinentNoise`)
//! - Multi-layered noise with terrain type selection
//! - Includes: base continents, hills, plains, continental shelf, ocean trenches
//! - More realistic and varied terrain
//! - Based on procedural planet generation techniques
//!
//! See `CONTINENT_GENERATION.md` for detailed documentation on how the system works.

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
    /// Maximum depth variation for oceans (positive value, applied negatively)
    pub ocean_depth_amplitude: f32,
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
            ocean_depth_amplitude: cfg.ocean_depth_amplitude,
        }
    }

    /// Sample the multi-octave continent noise at a given position
    /// Returns the final height value, incorporating continent and detail layers
    /// (Mountains come from tectonic plate simulation)
    pub fn sample_height(&self, position: Vec3) -> f32 {
        // Sample both noise layers (range roughly -1 to 1 due to amplitude)
        let continent_value = self.continent_scale.sample(position);
        let detail_value = self.detail_scale.sample(position);

        // Add detail to threshold for rough coastlines
        let adjusted_threshold = self.continent_threshold + (detail_value * 0.3);

        // Determine if this is continent or ocean based on threshold
        if continent_value > adjusted_threshold {
            // CONTINENT: Take the noise value above threshold and scale it
            // The noise naturally varies, creating elevation changes
            let height_above_threshold = continent_value - adjusted_threshold;

            // Scale by continent amplitude and add fine detail
            let base_height = height_above_threshold * self.continent_scale.amplitude;
            let detailed_height = base_height + (detail_value * self.detail_scale.amplitude);

            detailed_height.max(0.0) // Ensure non-negative for land
        } else {
            // OCEAN: Take the noise value below threshold and scale it negatively
            let depth_below_threshold = adjusted_threshold - continent_value;

            // Scale by ocean depth amplitude and add subtle detail
            let base_depth = depth_below_threshold * self.ocean_depth_amplitude;
            let detailed_depth = base_depth + (detail_value * self.detail_scale.amplitude * 0.3);

            -detailed_depth.max(0.0) // Make it negative for ocean depth
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



