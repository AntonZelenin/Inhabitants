//! # Continent Generation System
//!
//! This module provides two systems for generating planetary continents:
//!
//! ## Simple System (`ContinentNoiseConfig`)
//! - Two-layer noise: base continent shape + surface detail
//! - Fast and straightforward
//! - Good for testing and simple planets

use crate::config::NoiseConfig;
use glam::Vec3;

/// Multi-octave noise configuration for continent generation
pub struct ContinentNoiseConfig {
    /// Large-scale noise for continent placement (low frequency)
    pub continent_scale: NoiseConfig,
    /// Mid-scale noise for distorting continent shapes (domain warping)
    pub distortion_scale: NoiseConfig,
    /// Small-scale noise for coastline detail and local terrain variation
    pub detail_scale: NoiseConfig,
    /// Threshold for continent/ocean boundary (typically -0.3 to 0.3)
    pub continent_threshold: f32,
    /// Maximum depth variation for oceans (positive value, applied negatively)
    pub ocean_depth_amplitude: f32,
}

impl ContinentNoiseConfig {
    pub fn from_config(seed_base: u32, cfg: &crate::config::ContinentConfig) -> Self {
        Self {
            continent_scale: NoiseConfig::new(
                seed_base,
                cfg.continent_frequency,
                cfg.continent_amplitude,
            ),
            distortion_scale: NoiseConfig::new(
                seed_base.wrapping_add(1),
                cfg.distortion_frequency,
                cfg.distortion_amplitude,
            ),
            detail_scale: NoiseConfig::new(
                seed_base.wrapping_add(2),
                cfg.detail_frequency,
                cfg.detail_amplitude,
            ),
            continent_threshold: cfg.continent_threshold,
            ocean_depth_amplitude: cfg.ocean_depth_amplitude,
        }
    }

    /// Sample the multi-octave continent noise at a given position
    ///
    /// Returns the final height value, incorporating continent and detail layers
    /// (Mountains come from tectonic plate simulation)
    ///
    /// Uses 3-layer approach:
    /// 1. Base continent shape (large blobs)
    /// 2. Mid-scale distortion (domain warping to break up round shapes)
    /// 3. Fine detail (coastline roughness)
    pub fn sample_height(&self, position: Vec3) -> f32 {
        // Sample distortion noise for domain warping
        let distortion_value = self.distortion_scale.sample(position);

        // Create perpendicular vector for warping (tangent to sphere)
        // Use cross product with Y axis to get a consistent tangent direction
        // At poles (position parallel to Y), use X axis instead to avoid zero vector
        let cross = position.cross(Vec3::Y);
        let tangent = if cross.length_squared() > 1e-6 {
            cross.normalize()
        } else {
            // At poles, use X axis for cross product
            position.cross(Vec3::X).normalize_or_zero()
        };

        // Apply domain warping: offset the sampling position
        // This breaks up the round continent shapes into irregular forms
        let warp_offset = tangent * distortion_value;
        let warped_position = (position + warp_offset).normalize();

        // Sample continent at warped position (creates torn, irregular shapes)
        let continent_value = self.continent_scale.sample(warped_position);

        // Sample detail at original position (fine coastline roughness)
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
        // Apply same domain warping as in sample_height
        let distortion_value = self.distortion_scale.sample(position);
        let tangent = position.cross(Vec3::Y).normalize();
        let warp_offset = tangent * distortion_value;
        let warped_position = (position + warp_offset).normalize();

        let continent_value = self.continent_scale.sample(warped_position);
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
