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
            jungle: [0.05, 0.30, 0.05],
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

/// Smoothstep interpolation: sharper transitions than linear but still smooth.
fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Base biome color using smooth interpolation across temperature/precipitation space.
///
/// Each biome has a distinct core zone with narrow smoothstep transitions between
/// neighbors, preventing any single biome color from dominating its neighbors.
fn biome_base_color(temperature: f32, precipitation: f32, colors: &BiomeColors, th: &BiomeThresholds) -> [f32; 4] {
    let ice = rgb3_to_rgba(colors.ice);
    let tundra = rgb3_to_rgba(colors.tundra);
    let desert = rgb3_to_rgba(colors.desert);
    let savanna = rgb3_to_rgba(colors.savanna);
    let temperate = rgb3_to_rgba(colors.temperate);
    let jungle = rgb3_to_rgba(colors.jungle);

    let ice_tundra_range = th.tundra_temp - th.ice_temp;
    let tundra_boreal_range = th.boreal_temp - th.tundra_temp;
    let temperate_hot_range = th.hot_temp - th.temperate_temp;

    if temperature < th.ice_temp {
        ice
    } else if temperature < th.tundra_temp {
        // Ice -> Tundra
        let t = smoothstep(((temperature - th.ice_temp) / ice_tundra_range).clamp(0.0, 1.0));
        lerp_color(ice, tundra, t)
    } else if temperature < th.boreal_temp {
        // Tundra -> temperate/desert based on precipitation
        let t = smoothstep(((temperature - th.tundra_temp) / tundra_boreal_range).clamp(0.0, 1.0));
        let dry = lerp_color(tundra, desert, t);
        let wet = lerp_color(tundra, temperate, t);
        let p = smoothstep(precipitation.clamp(0.0, 1.0));
        lerp_color(dry, wet, p)
    } else if temperature < th.temperate_temp {
        // Temperate zone: desert vs forest based on precipitation
        let p = smoothstep(((precipitation - th.temperate_precip) / 0.4).clamp(0.0, 1.0));
        lerp_color(desert, temperate, p)
    } else if temperature < th.hot_temp {
        // Warm temperate: blend between temperate regime and hot regime by temperature.
        let temp_t = smoothstep(((temperature - th.temperate_temp) / temperate_hot_range).clamp(0.0, 1.0));

        // Temperate regime: desert ↔ temperate
        let p_temperate = smoothstep(((precipitation - th.temperate_precip) / 0.4).clamp(0.0, 1.0));
        let cool_color = lerp_color(desert, temperate, p_temperate);

        // Hot regime: desert → savanna → jungle
        let hot_color = hot_zone_color(precipitation, desert, savanna, jungle, th);

        lerp_color(cool_color, hot_color, temp_t)
    } else {
        // Hot zone: desert vs savanna vs jungle
        hot_zone_color(precipitation, desert, savanna, jungle, th)
    }
}

/// Hot-zone precipitation classification: desert → savanna → jungle with narrow transitions.
fn hot_zone_color(
    precipitation: f32,
    desert: [f32; 4],
    savanna: [f32; 4],
    jungle: [f32; 4],
    th: &BiomeThresholds,
) -> [f32; 4] {
    let p = precipitation.clamp(0.0, 1.0);
    let desert_savanna_width = (th.savanna_precip - th.desert_precip).max(0.01);
    let jungle_transition_width = 0.10;
    let jungle_start = th.jungle_precip;
    let jungle_end = jungle_start + jungle_transition_width;

    if p < th.desert_precip {
        desert
    } else if p < th.savanna_precip {
        let t = smoothstep(((p - th.desert_precip) / desert_savanna_width).clamp(0.0, 1.0));
        lerp_color(desert, savanna, t)
    } else if p < jungle_start {
        savanna
    } else if p < jungle_end {
        let t = smoothstep(((p - jungle_start) / jungle_transition_width).clamp(0.0, 1.0));
        lerp_color(savanna, jungle, t)
    } else {
        jungle
    }
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
