use crate::biome;
use crate::generator::cube_face_point;
use crate::planet::PlanetData;
use glam::Vec3;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Plates,
    Continents,
}

/// Raw mesh data that can be used by any rendering engine
#[derive(Debug, Clone)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
}

impl MeshData {
    /// Generate mesh data from planet data
    ///
    /// # Arguments
    /// * `planet` - The planet data to generate mesh from
    /// * `view_mode` - Whether to show plates or continents
    /// * `snow_threshold` - Height threshold above which snow appears (in continent view)
    /// * `continent_threshold` - Sea level threshold (dynamic from UI settings)
    pub fn from_planet(planet: &PlanetData, view_mode: ViewMode, snow_threshold: f32, continent_threshold: f32) -> Self {
        let size = planet.face_grid_size;
        let mut positions = Vec::new();
        let mut colors = Vec::new();
        let mut indices = Vec::new();
        let mut dir_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
        let mut vertex_indices = vec![vec![vec![0u32; size]; size]; 6];
        let mut next_index = 0u32;

        let quant_scale = (size - 1) as f32;

        // Generate vertices for all faces
        for (face_idx, face) in planet.faces.iter().enumerate() {
            for y in 0..size {
                let v = (y as f32 / (size - 1) as f32) * 2.0 - 1.0;
                for x in 0..size {
                    let u = (x as f32 / (size - 1) as f32) * 2.0 - 1.0;
                    let (nx, ny, nz) = cube_face_point(face_idx, u, v);
                    let dir = Vec3::new(nx, ny, nz).normalize();

                    let key = (
                        (dir.x * quant_scale).round() as i32,
                        (dir.y * quant_scale).round() as i32,
                        (dir.z * quant_scale).round() as i32,
                    );

                    let idx = *dir_map.entry(key).or_insert_with(|| {
                        let height = face.heightmap[y][x];
                        // Always render geometry at radius + height (including negative heights for ocean floor)
                        let radius = planet.radius + height;
                        let pos = dir * radius;
                        positions.push([pos.x, pos.y, pos.z]);

                        let color = calculate_vertex_color(
                            planet,
                            view_mode,
                            face_idx,
                            x,
                            y,
                            height,
                            dir,
                            snow_threshold,
                            continent_threshold,
                        );
                        colors.push(color);

                        let i = next_index;
                        next_index += 1;
                        i
                    });

                    vertex_indices[face_idx][y][x] = idx;
                }
            }
        }

        // Generate indices for all faces
        for face_idx in 0..6 {
            for y in 0..(size - 1) {
                for x in 0..(size - 1) {
                    let i0 = vertex_indices[face_idx][y][x];
                    let i1 = vertex_indices[face_idx][y][x + 1];
                    let i2 = vertex_indices[face_idx][y + 1][x];
                    let i3 = vertex_indices[face_idx][y + 1][x + 1];
                    indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
                }
            }
        }

        // Calculate normals
        let normals: Vec<[f32; 3]> = positions
            .iter()
            .map(|p| Vec3::from(*p).normalize().to_array())
            .collect();

        MeshData {
            positions,
            normals,
            colors,
            indices,
        }
    }
}

/// Calculate the color for a vertex based on view mode and planet properties
fn calculate_vertex_color(
    planet: &PlanetData,
    view_mode: ViewMode,
    face_idx: usize,
    x: usize,
    y: usize,
    height: f32,
    dir: Vec3,
    snow_threshold: f32,
    continent_threshold: f32,
) -> [f32; 4] {
    match view_mode {
        ViewMode::Plates => calculate_plate_view_color(planet, face_idx, x, y),
        ViewMode::Continents => calculate_continent_view_color(height, dir, snow_threshold, continent_threshold),
    }
}

/// Calculate color for plate view mode
fn calculate_plate_view_color(
    planet: &PlanetData,
    face_idx: usize,
    x: usize,
    y: usize,
) -> [f32; 4] {
    let plate_id = planet.plate_map[face_idx][y][x];
    let plate = &planet.plates[plate_id];
    let mut base_color = plate.debug_color;

    // Blend in boundary color if this is a boundary cell, with distance-based fade
    if let Some((boundary_color, opacity)) = planet.boundary_data.get_boundary_color(face_idx, x, y)
    {
        // Blend based on opacity: full boundary color at edges, fade to plate color
        base_color[0] = base_color[0] * (1.0 - opacity) + boundary_color[0] * opacity;
        base_color[1] = base_color[1] * (1.0 - opacity) + boundary_color[1] * opacity;
        base_color[2] = base_color[2] * (1.0 - opacity) + boundary_color[2] * opacity;
    }

    base_color
}

/// Calculate color for continent view mode
fn calculate_continent_view_color(
    height: f32,
    _dir: Vec3,
    snow_threshold: f32,
    continent_threshold: f32,
) -> [f32; 4] {
    // Calculate height relative to ocean level (which is at continent_threshold)
    let height_above_ocean = height - continent_threshold;
    
    if height_above_ocean > 0.0 {
        // Land (above ocean level)

        if height > snow_threshold {
            // Pure white snow above threshold
            [0.95, 0.95, 1.0, 1.0]
        } else if height > continent_threshold + (continent_threshold * 0.5) {
            // High elevation: Green mountains transitioning to snow
            let mountain_factor = ((height - (continent_threshold + continent_threshold * 0.5)) / (snow_threshold - (continent_threshold + continent_threshold * 0.5))).clamp(0.0, 1.0);
            // Interpolate from much darker green at mid-elevation to lighter green near snow
            [
                0.05 + mountain_factor * 0.2,     // Red: slight increase toward snow
                0.2 + mountain_factor * 0.2,      // Green: MUCH darker base green
                0.05 + mountain_factor * 0.15,    // Blue: low to keep it earthy green
                1.0,
            ]
        } else if height_above_ocean > continent_threshold * 0.05 {
            // Medium elevation: Transition from sandy shores to green mountains
            let transition_factor = ((height_above_ocean - continent_threshold * 0.05) / ((continent_threshold * 0.5) - (continent_threshold * 0.05))).clamp(0.0, 1.0);
            // Interpolate from light green/yellow to much darker forest green
            [
                0.4 - transition_factor * 0.35,   // Red: from light to very dark green
                0.5 - transition_factor * 0.3,    // Green: much darker transition
                0.15 - transition_factor * 0.1,   // Blue: earthy tone
                1.0,
            ]
        } else {
            // Low elevation near ocean level: Sandy/yellow shores
            let shore_factor = (height_above_ocean / (continent_threshold * 0.05)).clamp(0.0, 1.0);
            // Interpolate from sandy yellow at coast to light green inland
            [
                0.85 - shore_factor * 0.45,       // Red: sandy at coast, less inland
                0.75 - shore_factor * 0.25,       // Green: darker green at inland edge
                0.45 - shore_factor * 0.3,        // Blue: warm sandy tone
                1.0,
            ]
        }
    } else {
        // Ocean floor (below ocean level): sandy/light color visible through transparent ocean
        let depth = -height;
        let depth_factor = (depth / 1.0).clamp(0.0, 1.0);
        // Sandy light color
        [
            0.9 - depth_factor * 0.2,   // Red: sandy
            0.85 - depth_factor * 0.2,  // Green: sandy
            0.7 - depth_factor * 0.2,   // Blue: warm sandy color
            1.0,
        ]
    }
}

/// Calculate biome-based vertex colors for a planet mesh.
///
/// Called after temperature and precipitation cubemaps are ready,
/// to replace initial height-based colors with biome-aware colors.
pub fn calculate_biome_colors(
    positions: &[[f32; 3]],
    planet_radius: f32,
    continent_threshold: f32,
    snow_threshold: f32,
    land_temperature_bonus: f32,
    biome_colors: &biome::BiomeColors,
    biome_thresholds: &biome::BiomeThresholds,
    sample_temperature: impl Fn(Vec3) -> f32,
    sample_precipitation: impl Fn(Vec3) -> f32,
) -> Vec<[f32; 4]> {
    let ocean_level = planet_radius + continent_threshold;

    positions
        .iter()
        .map(|&[x, y, z]| {
            let position = Vec3::new(x, y, z);
            let direction = position.normalize();
            let vertex_radius = position.length();

            let height = vertex_radius - planet_radius;
            let height_above_ocean = height - continent_threshold;
            let is_land = vertex_radius > ocean_level;

            let base_temperature = sample_temperature(direction);
            let temperature = if is_land {
                base_temperature + land_temperature_bonus
            } else {
                base_temperature
            };
            let precipitation = sample_precipitation(direction);

            biome::biome_color(
                height_above_ocean,
                temperature,
                precipitation,
                height,
                snow_threshold,
                continent_threshold,
                biome_colors,
                biome_thresholds,
            )
        })
        .collect()
}
