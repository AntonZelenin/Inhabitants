pub const CONTINENTAL_FREQ: f32 = 3.0;
pub const CONTINENTAL_AMP: f32 = 0.7;
pub const OCEANIC_FREQ: f32 = CONTINENTAL_FREQ / 2.0;
pub const OCEANIC_AMP: f32 = CONTINENTAL_AMP / 10.0;

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