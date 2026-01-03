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
use noise::{NoiseFn, Fbm, Billow, Perlin, RidgedMulti, Seedable, MultiFractal};

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

// ============================================================================
// Advanced Continent Generation System
// ============================================================================

/// Advanced multi-layered continent noise generator
///
/// This system creates realistic planetary terrain using multiple noise layers:
/// - Base continent shapes with carved valleys
/// - **3× Turbulence warping for jagged coastlines and fjords**
/// - Terrain type selection (hills vs plains)
/// - Continental shelf with proper terracing
/// - Ocean depth variation with ridged trenches
///
/// The system is based on libnoise's complex planet example and adapted for
/// spherical cube-mapped surfaces. See `CONTINENT_GENERATION.md` for detailed documentation.
pub struct AdvancedContinentNoise {
    // Configuration
    sea_level: f64,
    shelf_level: f64,
    terrain_offset: f64,
    continent_height_scale: f64,
    ocean_depth_amplitude: f64,

    // Seed for all noise functions
    seed_base: u32,

    // Continent definition parameters (cached for runtime generation)
    continent_frequency: f64,
    continent_lacunarity: f64,
    hills_lacunarity: f64,
    plains_lacunarity: f64,
    detail_frequency: f64,
}

impl AdvancedContinentNoise {
    /// Create a new advanced continent noise generator from config
    pub fn new(seed_base: u32) -> Self {
        let config = crate::get_config();
        Self::from_config(seed_base, &config.continents)
    }

    /// Create from a ContinentConfig
    ///
    /// # Parameters
    /// - `seed_base`: Base seed for all noise functions
    /// - `cfg`: Configuration defining continent parameters
    pub fn from_config(seed_base: u32, cfg: &crate::config::ContinentConfig) -> Self {
        let sea_level = cfg.continent_threshold as f64;
        let shelf_level = cfg.shelf_level as f64;
        let continent_height_scale = (1.0 - sea_level) / 4.0;

        Self {
            sea_level,
            shelf_level,
            terrain_offset: cfg.terrain_offset as f64,
            continent_height_scale,
            ocean_depth_amplitude: cfg.ocean_depth_amplitude as f64,
            seed_base,
            continent_frequency: cfg.continent_frequency as f64,
            continent_lacunarity: cfg.continent_lacunarity as f64,
            hills_lacunarity: cfg.hills_lacunarity as f64,
            plains_lacunarity: cfg.plains_lacunarity as f64,
            detail_frequency: cfg.detail_frequency as f64,
        }
    }

    /// Sample the advanced continent noise at a 3D position
    ///
    /// This implements the full complex planet generation pipeline:
    /// 1. Base continent definition (FBM → curve → carver → min)
    /// 2. **3× Turbulence warping for jagged coastlines**
    /// 3. Select to apply warping only above sea level
    /// 4. Clamp to [-1, 1]
    /// 5. Terrain type definition with warping
    /// 6. Hills and plains generation
    /// 7. Terrain type selection and blending
    /// 8. Continental shelf with proper terracing
    /// 9. Ocean trenches with ridged noise
    ///
    /// # Parameters
    /// - `position`: Normalized 3D direction vector on sphere surface
    ///
    /// # Returns
    /// Final elevation value (negative = ocean, positive = land)
    pub fn sample_height(&self, position: Vec3) -> f32 {
        let pos_f64 = [position.x as f64, position.y as f64, position.z as f64];

        // ====================================================================
        // GROUP 1: BASE CONTINENT DEFINITION
        // ====================================================================

        // 1. Base continent FBM (14 octaves for detail)
        let continent_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base)
            .set_frequency(self.continent_frequency)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(14);

        let continent_value = continent_fbm.get(pos_f64);

        // 2. Apply curve to create continent profile
        let continent_curved = self.apply_curve(continent_value);

        // 3. Carver FBM to cut valleys
        let carver_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base + 1)
            .set_frequency(self.continent_frequency * 4.34375)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(11);

        let carver_value = carver_fbm.get(pos_f64);
        let carver_scaled = carver_value * 0.375 + 0.625;

        // 4. Min operation to carve
        let base_continent_def = continent_curved.min(carver_scaled);

        // ====================================================================
        // GROUP 2: CONTINENT DEFINITION WITH TURBULENCE WARPING
        // ====================================================================

        // This is the KEY step that was missing!
        // Apply 3× turbulence warping to create jagged, fjord-like coastlines

        // Turbulence 0: Large-scale warping
        let turb0_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base + 10)
            .set_frequency(self.continent_frequency * 15.25)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(6);

        let turb0_x = turb0_fbm.get([pos_f64[0], pos_f64[1], pos_f64[2]]);
        let turb0_y = turb0_fbm.get([pos_f64[0] + 100.0, pos_f64[1], pos_f64[2]]);
        let turb0_z = turb0_fbm.get([pos_f64[0], pos_f64[1] + 100.0, pos_f64[2]]);
        let turb0_power = 1.0 / (self.continent_frequency * 15.25 + 1.0);

        let warped0 = [
            pos_f64[0] + turb0_x * turb0_power,
            pos_f64[1] + turb0_y * turb0_power,
            pos_f64[2] + turb0_z * turb0_power,
        ];

        // Turbulence 1: Medium-scale warping
        let turb1_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base + 11)
            .set_frequency(self.continent_frequency * 47.25)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(6);

        let turb1_x = turb1_fbm.get(warped0);
        let turb1_y = turb1_fbm.get([warped0[0] + 100.0, warped0[1], warped0[2]]);
        let turb1_z = turb1_fbm.get([warped0[0], warped0[1] + 100.0, warped0[2]]);
        let turb1_power = 1.0 / (self.continent_frequency * 47.25 + 1.0);

        let warped1 = [
            warped0[0] + turb1_x * turb1_power,
            warped0[1] + turb1_y * turb1_power,
            warped0[2] + turb1_z * turb1_power,
        ];

        // Turbulence 2: Fine-scale warping
        let turb2_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base + 12)
            .set_frequency(self.continent_frequency * 95.25)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(6);

        let turb2_x = turb2_fbm.get(warped1);
        let turb2_y = turb2_fbm.get([warped1[0] + 100.0, warped1[1], warped1[2]]);
        let turb2_z = turb2_fbm.get([warped1[0], warped1[1] + 100.0, warped1[2]]);
        let turb2_power = 1.0 / (self.continent_frequency * 95.25 + 1.0);

        let warped2 = [
            warped1[0] + turb2_x * turb2_power,
            warped1[1] + turb2_y * turb2_power,
            warped1[2] + turb2_z * turb2_power,
        ];

        // Sample base continent at warped coordinates
        let warped_continent_value = continent_fbm.get(warped2);
        let warped_curved = self.apply_curve(warped_continent_value);
        let warped_carver = carver_fbm.get(warped2);
        let warped_carver_scaled = warped_carver * 0.375 + 0.625;
        let warped_continent = warped_curved.min(warped_carver_scaled);

        // Select: Apply warping only above sea level (this creates the jagged effect!)
        let continent_def = if base_continent_def > self.sea_level - 0.0625 {
            // Above or near sea level: use warped (jagged) continents
            let blend = ((base_continent_def - (self.sea_level - 0.0625)) / 0.125).clamp(0.0, 1.0);
            base_continent_def * (1.0 - blend) + warped_continent * blend
        } else {
            // Deep ocean: use smooth base
            base_continent_def
        };

        // Clamp continent def to [-1, 1] as in the example
        let continent_def_clamped = continent_def.clamp(-1.0, 1.0);

        // ====================================================================
        // GROUP 3: TERRAIN TYPE DEFINITION
        // ====================================================================

        let terrain_type_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base + 20)
            .set_frequency(self.continent_frequency * 18.125)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(3);

        let terrain_selector = terrain_type_fbm.get(pos_f64) + self.terrain_offset;

        // ====================================================================
        // GROUP 4-5: HILLS AND PLAINS
        // ====================================================================

        let hills_billow = Billow::<Perlin>::default()
            .set_seed(self.seed_base + 30)
            .set_frequency(self.detail_frequency * 1.0)
            .set_persistence(0.5)
            .set_lacunarity(self.hills_lacunarity)
            .set_octaves(6);

        let plains_billow = Billow::<Perlin>::default()
            .set_seed(self.seed_base + 40)
            .set_frequency(self.detail_frequency * 0.5)
            .set_persistence(0.5)
            .set_lacunarity(self.plains_lacunarity)
            .set_octaves(4);

        let hills_value = hills_billow.get(pos_f64);
        let plains_value = plains_billow.get(pos_f64);

        // ====================================================================
        // GROUP 8-10: SCALED TERRAIN
        // ====================================================================

        let scaled_hills = hills_value * 0.125;
        let scaled_plains = plains_value * 0.0625;

        // ====================================================================
        // GROUP 12: FINAL PLANET ASSEMBLY
        // ====================================================================

        let mut final_elevation = continent_def_clamped * self.continent_height_scale;

        // Add terrain based on elevation and terrain type
        if continent_def_clamped > self.sea_level {
            // LAND: Add hills or plains based on terrain selector
            if terrain_selector > 0.5 {
                final_elevation += scaled_hills;
            } else {
                final_elevation += scaled_plains;
            }
        } else if continent_def_clamped > self.shelf_level {
            // CONTINENTAL SHELF: Apply terracing
            final_elevation = self.apply_continental_shelf(continent_def_clamped);
        } else {
            // DEEP OCEAN: Add trenches
            let ocean_trenches = RidgedMulti::<Perlin>::default()
                .set_seed(self.seed_base + 50)
                .set_frequency(self.continent_frequency * 4.375)
                .set_lacunarity(self.continent_lacunarity)
                .set_octaves(16);

            let trench_value = ocean_trenches.get(pos_f64);
            let trench_depth = (trench_value * self.ocean_depth_amplitude * 0.25).abs();

            final_elevation = continent_def_clamped - trench_depth;
        }

        final_elevation.clamp(-2.0, 2.0) as f32
    }

    /// Apply curve transformation to reshape continent profiles
    ///
    /// Uses linear interpolation between control points to create
    /// custom elevation response curves.
    fn apply_curve(&self, value: f64) -> f64 {
        // Curve control points to shape continent profiles
        let sea_level = self.sea_level;
        let continent_curve = vec![
            (-2.0 + sea_level, -1.625 + sea_level),
            (-1.0 + sea_level, -1.375 + sea_level),
            (0.0 + sea_level, -0.375 + sea_level),
            (0.0625 + sea_level, 0.125 + sea_level),
            (0.125 + sea_level, 0.25 + sea_level),
            (0.25 + sea_level, 1.0 + sea_level),
            (0.5 + sea_level, 0.25 + sea_level),
            (0.75 + sea_level, 0.25 + sea_level),
            (1.0 + sea_level, 0.5 + sea_level),
            (2.0 + sea_level, 0.5 + sea_level),
        ];

        // Find the two control points that bracket this value
        for i in 0..continent_curve.len() - 1 {
            let (x0, y0) = continent_curve[i];
            let (x1, y1) = continent_curve[i + 1];

            if value >= x0 && value <= x1 {
                // Linear interpolation between control points
                let t = (value - x0) / (x1 - x0);
                return y0 + t * (y1 - y0);
            }
        }

        // If outside range, clamp to nearest control point
        if value < continent_curve[0].0 {
            continent_curve[0].1
        } else {
            continent_curve.last().unwrap().1
        }
    }

    /// Apply continental shelf terracing
    ///
    /// Creates stepped elevations between deep ocean and coastline,
    /// matching the libnoise example's terrace implementation.
    fn apply_continental_shelf(&self, value: f64) -> f64 {
        // Terrace control points: -1.0, -0.75, shelf_level, sea_level, 1.0
        let terrace_points = vec![
            -1.0,
            -0.75,
            self.shelf_level,
            self.sea_level,
            1.0,
        ];

        // Find which segment we're in
        for i in 0..terrace_points.len() - 1 {
            let p0 = terrace_points[i];
            let p1 = terrace_points[i + 1];

            if value >= p0 && value <= p1 {
                // Create terrace step with some smoothing
                let t = (value - p0) / (p1 - p0);
                let curve_t = t * t * (3.0 - 2.0 * t); // Smoothstep
                return p0 + curve_t * (p1 - p0) * 0.5; // Flatten the steps
            }
        }

        value
    }

    /// Get just the continent mask (0.0 = ocean, 1.0 = continent)
    ///
    /// Useful for visualization and debugging.
    pub fn sample_continent_mask(&self, position: Vec3) -> f32 {
        let pos_f64 = [position.x as f64, position.y as f64, position.z as f64];

        // Use the same base continent FBM as in sample_height
        let continent_fbm = Fbm::<Perlin>::default()
            .set_seed(self.seed_base)
            .set_frequency(self.continent_frequency)
            .set_persistence(0.5)
            .set_lacunarity(self.continent_lacunarity)
            .set_octaves(14);

        let continent_value = continent_fbm.get(pos_f64);

        if continent_value > self.sea_level {
            ((continent_value - self.sea_level) / (1.0 - self.sea_level))
                .clamp(0.0, 1.0) as f32
        } else {
            0.0
        }
    }
}

