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
    pub fn from_planet(planet: &PlanetData, view_mode: ViewMode, snow_threshold: f32) -> Self {
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
) -> [f32; 4] {
    match view_mode {
        ViewMode::Plates => calculate_plate_view_color(planet, face_idx, x, y),
        ViewMode::Continents => calculate_continent_view_color(planet, height, dir, snow_threshold),
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
    planet: &PlanetData,
    height: f32,
    dir: Vec3,
    snow_threshold: f32,
) -> [f32; 4] {
    let continent_mask = planet.continent_noise.sample_continent_mask(dir);

    if height > 0.0 {
        // Land (above sea level): green to brown gradient, with snow caps at high elevation
        let height_factor = (height / 1.0).clamp(0.0, 1.0);

        if height > snow_threshold {
            // Pure white snow above threshold
            [0.95, 0.95, 1.0, 1.0]
        } else {
            // Regular land (green to brown)
            let green_base = 0.4 + continent_mask * 0.2;
            [
                0.2 + height_factor * 0.5,         // Red: browns at high elevation
                green_base - height_factor * 0.15, // Green: less at height
                0.1,                               // Blue: low
                1.0,
            ]
        }
    } else {
        // Ocean (below sea level): pure blue gradient based on depth
        let depth = -height;
        let depth_factor = (depth / 1.0).clamp(0.0, 1.0);
        [
            0.0,                      // Red: none
            0.0,                      // Green: none (pure blue!)
            0.4 + depth_factor * 0.4, // Blue: nice blue, darker with depth
            1.0,
        ]
    }
}
