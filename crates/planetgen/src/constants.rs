use std::ops::Range;

pub const CELLS_PER_UNIT: f32 = 3.0;
pub const CONTINENTAL_FREQ: f32 = 3.0;
pub const CONTINENTAL_AMP: f32 = 0.7;
pub const OCEANIC_FREQ: f32 = CONTINENTAL_FREQ / 2.0;
pub const OCEANIC_AMP: f32 = CONTINENTAL_AMP / 10.0;

pub const PLANET_MIN_RADIUS: f32 = 30.0;
pub const PLANET_MAX_RADIUS: f32 = 80.0;

// Frequency controls how wide the bends are: lower freq = big sweeping arcs, higher freq = more jagged.
pub const PLATE_BOUNDARY_DISTORTION_FREQUENCY: f32 = 7.0;
// Amplitude controls how “wiggly” the boundaries get: 0.1–0.3 is usually enough.
pub const PLATE_BOUNDARY_DISTORTION_AMPLITUDE: f32 = 0.2;
// raw noise gives -1.0..1.0, which will lead to huge warps for dozens of degrees
// 0.05–0.1: subtle wavy boundaries.
// 0.3–0.5: wild distortions, noisy patches.
pub const PLATE_BOUNDARY_WARP_MULTIPLIER: f32 = 0.2;

// 0.3-0.5: Tight packing, some elongation risk
// 0.6-0.8: Good balance (current: 0.8)
// 0.9-1.2: Well-spaced, robust against elongation
// 1.3-1.5: Very spread out
// 1.6+: Too restrictive, may not converge
pub const MIN_PLATE_SEPARATION_CHORD_DISTANCE: f32 = 0.5;

// probability for a plate to be continental vs oceanic
pub const CONTINENTAL_PLATE_PROBABILITY: f64 = 0.5;
// this constant helps microplates win cells, if we use 1.0 instead, we won't have
// any microplates, I don't know how it works though
pub const MICRO_PLATE_WEIGHT_FACTOR: f32 = 2.7;

// Microplate generation constants
pub const MICRO_PLATE_FREQUENCY_MULTIPLIER: f32 = 1.5;
pub const MICRO_PLATE_AMPLITUDE_MULTIPLIER: f32 = 0.3;
pub const MICRO_PLATE_JITTER_RANGE: Range<f32> = -0.1..0.1;

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
