/// Biome classification and coloring based on temperature and precipitation.
///
/// Produces smooth color gradients between biome zones rather than hard boundaries.
/// Mountain snow (based on height threshold) and ocean floor coloring are preserved.

/// Configurable biome zone thresholds.
#[derive(Clone, Debug)]
pub struct BiomeThresholds {
    // Temperature thresholds (°C) — define biome zone boundaries
    pub ice_temp: f32,
    pub tundra_temp: f32,
    pub boreal_temp: f32,
    pub temperate_temp: f32,
    pub hot_temp: f32,
    // Precipitation thresholds (0.0-1.0) — control dry↔wet biome boundaries
    pub desert_precip: f32,
    pub savanna_precip: f32,
    pub jungle_precip: f32,
    pub temperate_precip: f32,
}

impl Default for BiomeThresholds {
    fn default() -> Self {
        Self {
            ice_temp: -10.0,
            tundra_temp: 0.0,
            boreal_temp: 5.0,
            temperate_temp: 15.0,
            hot_temp: 20.0,
            desert_precip: 0.15,
            savanna_precip: 0.25,
            jungle_precip: 0.45,
            temperate_precip: 0.1,
        }
    }
}

/// Configurable biome colors. Each color is RGB in [0.0, 1.0].
#[derive(Clone, Debug)]
pub struct BiomeColors {
    pub ice: [f32; 3],
    pub tundra: [f32; 3],
    pub desert: [f32; 3],
    pub savanna: [f32; 3],
    pub temperate: [f32; 3],
    pub jungle: [f32; 3],
}

impl Default for BiomeColors {
    fn default() -> Self {
        Self {
            ice: [0.85, 0.90, 0.95],
            tundra: [0.55, 0.60, 0.50],
            desert: [0.82, 0.72, 0.45],
            savanna: [0.60, 0.65, 0.25],
            temperate: [0.15, 0.40, 0.10],
            jungle: [0.0, 0.2, 0.0],
        }
    }
}

/// Compute the biome-based RGBA color for a vertex.
pub fn biome_color(
    height_above_ocean: f32,
    temperature: f32,
    precipitation: f32,
    height: f32,
    snow_threshold: f32,
    continent_threshold: f32,
    colors: &BiomeColors,
    thresholds: &BiomeThresholds,
) -> [f32; 4] {
    // Ocean floor: sandy color
    if height_above_ocean <= 0.0 {
        let depth = -height;
        let depth_factor = (depth / 1.0).clamp(0.0, 1.0);
        return [
            0.9 - depth_factor * 0.2,
            0.85 - depth_factor * 0.2,
            0.7 - depth_factor * 0.2,
            1.0,
        ];
    }

    // Mountain snow: pure white above threshold
    if height > snow_threshold {
        return [0.95, 0.95, 1.0, 1.0];
    }

    // Smooth transition zone near snow line (top 15% of mountain range)
    let snow_transition_start = snow_threshold - (snow_threshold - continent_threshold) * 0.15;
    let snow_blend = if height > snow_transition_start {
        ((height - snow_transition_start) / (snow_threshold - snow_transition_start)).clamp(0.0, 1.0)
    } else {
        0.0
    };

    // Narrow sandy strip at coast
    let shore_width = continent_threshold * 0.05;
    let shore_blend = if height_above_ocean < shore_width {
        1.0 - (height_above_ocean / shore_width).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let shore_color: [f32; 4] = [0.85, 0.75, 0.45, 1.0];

    // Base biome color from temperature and precipitation
    let base_color = biome_base_color(temperature, precipitation, colors, thresholds);

    // Blend with shore
    let mut color = lerp_color(base_color, shore_color, shore_blend);

    // Blend with snow near mountain tops
    let snow_color: [f32; 4] = [0.95, 0.95, 1.0, 1.0];
    color = lerp_color(color, snow_color, snow_blend);

    color
}

fn rgb3_to_rgba(c: [f32; 3]) -> [f32; 4] {
    [c[0], c[1], c[2], 1.0]
}

/// Compute Gaussian weights for each biome based on distance in climate space.
///
/// Returns weights for [ice, tundra, desert, savanna, temperate, jungle].
/// Ice and tundra are precipitation-independent (any precipitation is fine).
fn biome_weights(temperature: f32, precipitation: f32, th: &BiomeThresholds) -> [f32; 6] {
    // Derive biome centers from thresholds
    let ice_center_temp = th.ice_temp - 5.0;
    let tundra_center_temp = (th.ice_temp + th.boreal_temp) / 2.0;
    let temperate_center_temp = (th.boreal_temp + th.hot_temp) / 2.0;
    let hot_center_temp = th.hot_temp + 5.0;

    let desert_center_precip = th.desert_precip / 2.0;
    let savanna_center_precip = (th.desert_precip + th.jungle_precip) / 2.0;
    let temperate_center_precip = (th.temperate_precip + th.jungle_precip) / 2.0;
    let jungle_center_precip = th.jungle_precip + 0.15;

    // Derive spreads from threshold spacing
    let ice_temp_spread = (th.tundra_temp - th.ice_temp).abs().max(3.0);
    let tundra_temp_spread = (th.boreal_temp - th.ice_temp).abs().max(3.0) / 2.0 + 2.0;
    let desert_temp_spread = (th.hot_temp - th.boreal_temp).abs().max(3.0);
    let savanna_temp_spread = (th.hot_temp - th.temperate_temp).abs().max(3.0);
    let temperate_temp_spread = (th.hot_temp - th.boreal_temp).abs().max(3.0) / 2.0 + 2.0;
    let jungle_temp_spread = (th.hot_temp - th.temperate_temp).abs().max(3.0);

    let desert_precip_spread = th.desert_precip.max(0.05) + 0.05;
    let savanna_precip_spread = (th.jungle_precip - th.desert_precip).abs().max(0.05) / 2.0 + 0.05;
    let temperate_precip_spread = 0.25;
    let jungle_precip_spread = 0.2;

    // Helper: Gaussian weight with both temp and precip terms
    let gaussian_tp = |ct: f32, st: f32, cp: f32, sp: f32| -> f32 {
        let dt = (temperature - ct) / st;
        let dp = (precipitation - cp) / sp;
        (-0.5 * (dt * dt + dp * dp)).exp()
    };

    // Helper: Gaussian weight with temp only (precip-independent)
    let gaussian_t = |ct: f32, st: f32| -> f32 {
        let dt = (temperature - ct) / st;
        (-0.5 * dt * dt).exp()
    };

    [
        gaussian_t(ice_center_temp, ice_temp_spread),
        gaussian_t(tundra_center_temp, tundra_temp_spread),
        gaussian_tp(hot_center_temp, desert_temp_spread, desert_center_precip, desert_precip_spread),
        gaussian_tp(hot_center_temp, savanna_temp_spread, savanna_center_precip, savanna_precip_spread),
        gaussian_tp(temperate_center_temp, temperate_temp_spread, temperate_center_precip, temperate_precip_spread),
        gaussian_tp(hot_center_temp, jungle_temp_spread, jungle_center_precip, jungle_precip_spread),
    ]
}

/// Base biome color using Gaussian weight blending across temperature/precipitation space.
///
/// Each biome has a center point in climate space with spread values controlling
/// its influence zone. Colors are blended using normalized Gaussian weights,
/// producing soft, organic transitions between biomes.
fn biome_base_color(temperature: f32, precipitation: f32, colors: &BiomeColors, th: &BiomeThresholds) -> [f32; 4] {
    let biome_colors = [
        rgb3_to_rgba(colors.ice),
        rgb3_to_rgba(colors.tundra),
        rgb3_to_rgba(colors.desert),
        rgb3_to_rgba(colors.savanna),
        rgb3_to_rgba(colors.temperate),
        rgb3_to_rgba(colors.jungle),
    ];

    let weights = biome_weights(temperature, precipitation, th);
    let total: f32 = weights.iter().sum();

    if total < 1e-10 {
        // Fallback: if all weights are near zero, use temperate
        return rgb3_to_rgba(colors.temperate);
    }

    let mut result = [0.0f32; 4];
    for (i, &w) in weights.iter().enumerate() {
        let nw = w / total;
        for c in 0..4 {
            result[c] += nw * biome_colors[i][c];
        }
    }
    result
}

/// Linear interpolation between two RGBA colors.
fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}
