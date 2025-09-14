use std::ops::Range;

// cells per unit influences the performance of the planet generation quite a lot
pub const CELLS_PER_UNIT: f32 = 5.0;
pub const CONTINENTAL_FREQ: f32 = 3.0;
pub const CONTINENTAL_AMP: f32 = 0.7;
pub const OCEANIC_FREQ: f32 = CONTINENTAL_FREQ / 2.0;
pub const OCEANIC_AMP: f32 = CONTINENTAL_AMP / 10.0;

pub const PLANET_MIN_RADIUS: f32 = 30.0;
pub const PLANET_MAX_RADIUS: f32 = 80.0;

pub const DEFAULT_NUM_PLATES: usize = 7;
pub const DEFAULT_NUM_MICRO_PLATES: usize = 6;

/// Frequency controls how wide the bends are: lower freq = big sweeping arcs, higher freq = more jagged.
pub const PLATE_BOUNDARY_DISTORTION_FREQUENCY: f32 = 7.0;
/// Amplitude controls how “wiggly” the boundaries get: 0.1–0.3 is usually enough.
pub const PLATE_BOUNDARY_DISTORTION_AMPLITUDE: f32 = 0.2;
/// raw noise gives -1.0..1.0, which will lead to huge warps for dozens of degrees
/// 0.05–0.1: subtle wavy boundaries.
/// 0.3–0.5: wild distortions, noisy patches.
pub const PLATE_BOUNDARY_WARP_MULTIPLIER: f32 = 0.2;

/// Spatial frequency of the flow field used to bend plate boundaries.
/// Lower values produce larger, smoother swirls; higher values add finer detail.
/// Examples: 0.15–0.40 = broad, plate-scale bends; 0.5–1.0 = medium; >1.5 = busy/jittery.
pub const DEFAULT_FLOW_WARP_FREQ: f32 = 0.8;
/// Actually doesn't influence anything
/// Strength of the flow vectors before projection to the tangent plane.
/// Controls how hard each step pushes. Examples: 0.10 subtle, 0.25 balanced (default), 0.40 strong, >0.60 chaotic.
pub const DEFAULT_FLOW_WARP_AMP: f32 = 0.25;
/// Number of advection steps applied per cell. More steps = more coherent, larger-scale displacement (but slower).
/// Examples: 1 minimal, 2–4 typical, 5–8 heavy warp.
pub const DEFAULT_FLOW_WARP_STEPS: usize = 2;
/// Angular step size per advection step (radians). Sets the along-surface distance moved each step.
/// Examples: 0.05 (~3°) subtle, 0.12 (~7°) default, 0.25 (~14°) strong, >0.50 (~29°) extreme.
pub const DEFAULT_FLOW_WARP_STEP_ANGLE: f32 = 0.1;

/// 0.3-0.5: Tight packing, some elongation risk
/// 0.6-0.8: Good balance (current: 0.8)
/// 0.9-1.2: Well-spaced, robust against elongation
/// 1.3-1.5: Very spread out
/// 1.6+: Too restrictive, may not converge
pub const MIN_PLATE_SEPARATION_CHORD_DISTANCE: f32 = 0.5;

/// Probability for a plate to be continental vs oceanic
pub const CONTINENTAL_PLATE_PROBABILITY: f64 = 0.5;
/// This constant helps microplates win cells, if we use 1.0 instead, we won't have
/// any microplates, I don't know how it works though
pub const MICRO_PLATE_WEIGHT_FACTOR: f32 = 2.7;

// Microplate generation constants
pub const MICRO_PLATE_FREQUENCY_MULTIPLIER: f32 = 1.5;
pub const MICRO_PLATE_AMPLITUDE_MULTIPLIER: f32 = 0.3;
pub const MICRO_PLATE_JITTER_RANGE: Range<f32> = -0.1..0.1;

/// Probability that a plate will be selected as a primary for merging (10%)
pub const PLATE_MERGE_SELECTION_PROBABILITY: f64 = 0.07;
/// Probability of selecting 2 neighbors instead of 1 when merging (30%)
pub const PLATE_MERGE_TWO_NEIGHBORS_PROBABILITY: f64 = 0.2;

pub const DEBUG_COLORS: &[[f32; 4]] = &[
    [1.0, 0.0, 0.0, 1.0], // red
    [0.0, 1.0, 0.0, 1.0], // green
    [0.0, 0.0, 1.0, 1.0], // blue
    [1.0, 1.0, 0.0, 1.0], // yellow
    [1.0, 0.0, 1.0, 1.0], // magenta
    [0.0, 1.0, 1.0, 1.0], // cyan
    [1.0, 0.5, 0.0, 1.0], // orange
    [0.5, 0.0, 1.0, 1.0], // violet
    [0.0, 0.5, 1.0, 1.0], // sky blue
    [0.5, 1.0, 0.0, 1.0], // lime
];
