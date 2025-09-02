use crate::config::NoiseConfig;
use crate::constants::*;
use crate::planet::*;
use crate::plate::TectonicPlate;
use glam::Vec3;
use rand::{random_bool, random_range};
use std::collections::HashMap;

// 0.3-0.5: Tight packing, some elongation risk
// 0.6-0.8: Good balance (current: 0.8)
// 0.9-1.2: Well-spaced, robust against elongation
// 1.3-1.5: Very spread out
// 1.6+: Too restrictive, may not converge
pub const MIN_PLATE_SEPARATION_CHORD_DISTANCE: f32 = 0.5;
pub const STRIDE: usize = 1;

#[derive(Debug, Clone)]
pub struct PlanetSettings {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
}

pub fn generate(settings: PlanetSettings) -> PlanetData {
    // Each cube face represents a square section of the unit sphere, scaled the planet's radius.
    // cells_per_unit = how many grid cells per 1 unit of world space
    // radius * cells_per_unit = number of cells from edge to edge on one face
    // +1 = adds 1 to include both start and end of the grid (for vertices, not just quads)
    let face_grid_size = (settings.radius * settings.cells_per_unit).ceil() as usize + 1;

    let mut plates = generate_plates(settings.num_plates);
    let mut plate_map = assign_plates(face_grid_size, &plates);

    let micros = generate_microplates(
        face_grid_size,
        &plates,
        &plate_map,
        settings.num_micro_plates,
    );
    plates.extend(micros);

    plate_map = assign_plates(face_grid_size, &plates);

    let faces = generate_faces(face_grid_size, &plates, &plate_map);
    PlanetData {
        faces,
        face_grid_size,
        radius: settings.radius,
        plate_map,
        plates,
    }
}

fn make_plate(
    id: usize,
    direction: Vec3,
    plate_type: PlateType,
    size_class: PlateSizeClass,
    freq: f32,
    amp: f32,
) -> TectonicPlate {
    let noise_seed = random_range(0_u32..u32::MAX);
    let color = DEBUG_COLORS[id % DEBUG_COLORS.len()];
    TectonicPlate {
        id,
        direction,
        plate_type,
        size_class,
        noise_config: NoiseConfig::new(noise_seed, freq, amp),
        debug_color: color,
    }
}

/// Generates the main tectonic plates for the planet
///
/// Creates random continental and oceanic plates with appropriate noise parameters.
/// Each plate gets a random seed direction on the unit sphere
fn generate_plates(num_plates: usize) -> Vec<TectonicPlate> {
    let mut directions: Vec<Vec3> = (0..num_plates)
        .map(|_| {
            Vec3::new(
                random_range(-1.0..1.0),
                random_range(-1.0..1.0),
                random_range(-1.0..1.0),
            )
            .normalize()
        })
        .collect();

    enforce_minimum_plate_distance(&mut directions);

    directions
        .into_iter()
        .enumerate()
        .map(|(id, direction)| {
            let plate_type = if random_bool(0.5) {
                PlateType::Continental
            } else {
                PlateType::Oceanic
            };
            let (freq, amp) = match plate_type {
                PlateType::Continental => (CONTINENTAL_FREQ, CONTINENTAL_AMP),
                PlateType::Oceanic => (OCEANIC_FREQ, OCEANIC_AMP),
            };
            make_plate(
                id,
                direction,
                plate_type,
                PlateSizeClass::Regular,
                freq,
                amp,
            )
        })
        .collect()
}

/// Iteratively enforces minimum distance between tectonic plate centers.
///
/// Uses a relaxation algorithm to move plates apart when they're too close.
/// Continues until all plates meet the minimum distance requirement or max iterations reached.
///
/// # Complexity
/// `O(P² · I)`, where `P` is the number of plates and `I` is the number of iterations (<= max_iterations).
///
/// # Notes
/// - Inputs should be unit vectors; the function re-normalises after each relaxation step.
/// - Uses chord distance on unit sphere scaled by radius for intuitive distance calculations.
fn enforce_minimum_plate_distance(directions: &mut Vec<Vec3>) {
    let max_iterations = 50;
    let eps = 1e-6_f32;

    for _ in 0..max_iterations {
        let mut any_moved = false;
        let mut adjustments = vec![Vec3::ZERO; directions.len()];

        // Calculate position adjustments between all pairs of plates
        for i in 0..directions.len() {
            for j in (i + 1)..directions.len() {
                let dir_i = directions[i];
                let dir_j = directions[j];

                // Calculate chord distance on unit sphere surface
                let dot = dir_i.dot(dir_j).clamp(-1.0, 1.0);
                let chord_distance = (2.0 * (1.0 - dot)).sqrt();

                // If too close, calculate position adjustments
                if chord_distance < MIN_PLATE_SEPARATION_CHORD_DISTANCE {
                    any_moved = true;

                    // Calculate the vector between the two points
                    let diff = dir_j - dir_i;
                    let diff_length = diff.length();

                    if diff_length > eps {
                        let distance_deficit = MIN_PLATE_SEPARATION_CHORD_DISTANCE - chord_distance;
                        // Each plate moves half the distance needed to meet the criteria
                        let adjustment_magnitude = distance_deficit * 0.5;
                        let diff_normalized = diff / diff_length;

                        // Apply adjustments to both plates (equal and opposite)
                        adjustments[i] -= diff_normalized * adjustment_magnitude;
                        adjustments[j] += diff_normalized * adjustment_magnitude;
                    }
                }
            }
        }

        // Apply position adjustments and re-normalize to sphere surface
        for i in 0..directions.len() {
            if adjustments[i].length_squared() > eps * eps {
                directions[i] = (directions[i] + adjustments[i]).normalize();
            }
        }

        // If no plates moved significantly, we're done
        if !any_moved {
            break;
        }
    }
}

/// Generates smaller microplates along the boundaries of major plates
///
/// Microplates are placed at locations where different major plates meet,
/// creating more detailed terrain features along plate boundaries.
fn generate_microplates(
    face_grid_size: usize,
    plates: &[TectonicPlate],
    plate_map: &PlateMap,
    num_micro_plates: usize,
) -> Vec<TectonicPlate> {
    (0..num_micro_plates)
        .map(|i| {
            let id = plates.len() + i;
            let (f, x, y) = loop {
                let f = random_range(0..6);
                let y = random_range(0..face_grid_size);
                let x = random_range(0..face_grid_size);
                let c = plate_map[f][y][x];
                let r = plate_map[f][y][(x + 1).min(face_grid_size - 1)];
                let d = plate_map[f][(y + 1).min(face_grid_size - 1)][x];
                if r != c || d != c {
                    break (f, x, y);
                }
            };
            let (dx, dy, dz) = cube_face_point(
                f,
                x as f32 * 2.0 / (face_grid_size as f32 - 1.0) - 1.0,
                y as f32 * 2.0 / (face_grid_size as f32 - 1.0) - 1.0,
            );
            let base_dir = Vec3::new(dx, dy, dz).normalize();
            // *tiny* jitter so seed stays close to boundary
            let jitter = Vec3::new(
                random_range(-0.1..0.1),
                random_range(-0.1..0.1),
                random_range(-0.1..0.1),
            );
            let seed_dir = (base_dir + jitter).normalize();
            // smaller scale noise
            let freq = CONTINENTAL_FREQ * 1.5;
            let amp = CONTINENTAL_AMP * 0.3;
            make_plate(
                id,
                seed_dir,
                PlateType::Continental,
                PlateSizeClass::Micro,
                freq,
                amp,
            )
        })
        .collect()
}

/// Assigns a plate ID to every cell on each cube face by:
///
/// The planet is represented as a cube with 6 faces. Each face is divided into a grid.
/// Each tectonic plate has a direction pointing from the center of the planet to
/// somewhere on its surface.
///
/// For every grid cell on every cube face:
/// - take the (x,y) coordinates on the cube face and convert them to a 3D direction vector
///   pointing from planet center to that surface point;
/// - compare this grid cell's direction with ALL tectonic plates' direction vectors.
/// - the plate whose direction is closest (smallest angular distance) "wins" that grid cell
/// - store the winner: Put that winning plate's ID into map[face][y][x]
fn assign_plates(face_grid_size: usize, plates: &[TectonicPlate]) -> PlateMap {
    let mut map = vec![vec![vec![0; face_grid_size]; face_grid_size]; 6];

    let pre: Vec<(Vec3, f32, usize)> = plates
        .iter()
        .map(|p| {
            let w = match p.size_class {
                PlateSizeClass::Regular => 1.0,
                PlateSizeClass::Micro => 2.7,
            };
            (p.direction.normalize(), w * w, p.id)
        })
        .collect();

    let inv = 1.0 / (face_grid_size as f32 - 1.0);
    for f in 0..6 {
        for y in 0..face_grid_size {
            let v = y as f32 * inv * 2.0 - 1.0;
            for x in 0..face_grid_size {
                let u = x as f32 * inv * 2.0 - 1.0;
                let dir = Vec3::from(cube_face_point(f, u, v)).normalize();
                let mut best_id = 0usize;
                let mut best_score = f32::INFINITY;
                for (pdir, w2, pid) in &pre {
                    let dot = dir.dot(*pdir).clamp(-1.0, 1.0);
                    let score = w2 * (1.0 - dot);
                    if score < best_score {
                        best_score = score;
                        best_id = *pid;
                    }
                }
                map[f][y][x] = best_id;
            }
        }
    }
    map
}

/// Generates heightmaps for all six cube faces of the planet
///
/// For each face, samples the noise function of the assigned tectonic plate
/// to create terrain height values at each grid point.
fn generate_faces(
    face_grid_size: usize,
    plates: &[TectonicPlate],
    plate_map: &PlateMap,
) -> [CubeFace; 6] {
    let blank = CubeFace {
        heightmap: vec![vec![0.0; face_grid_size]; face_grid_size],
    };
    let mut faces = [
        blank.clone(),
        blank.clone(),
        blank.clone(),
        blank.clone(),
        blank.clone(),
        blank.clone(),
    ];
    for face_idx in 0..6 {
        for y in 0..face_grid_size {
            let v = y as f32 / (face_grid_size - 1) as f32 * 2.0 - 1.0;
            for x in 0..face_grid_size {
                let u = x as f32 / (face_grid_size - 1) as f32 * 2.0 - 1.0;
                let dir = Vec3::from(cube_face_point(face_idx, u, v)).normalize();
                let plate_id = plate_map[face_idx][y][x];
                let height = plates[plate_id].noise_config.sample(dir);
                faces[face_idx].heightmap[y][x] = height;
            }
        }
    }
    faces
}

/// Converts 2D cube face coordinates to 3D world coordinates
///
/// Maps normalized coordinates (u, v) in range [-1, 1] on a specific cube face
/// to 3D coordinates on the unit cube surface.
pub fn cube_face_point(face_idx: usize, u: f32, v: f32) -> (f32, f32, f32) {
    match face_idx {
        0 => (1.0, v, -u),
        1 => (-1.0, v, u),
        2 => (u, 1.0, -v),
        3 => (u, -1.0, v),
        4 => (u, v, 1.0),
        5 => (-u, v, -1.0),
        _ => (0.0, 0.0, 0.0),
    }
}

/// A fast hash function (SplitMix64) for pseudo-random uniform distribution.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E3779B97F4A7C15);
    let mut r = z;
    r = (r ^ (r >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    r = (r ^ (r >> 27)).wrapping_mul(0x94D049BB133111EB);
    r ^ (r >> 31)
}

/// Hashes a single cell uniquely given its face, coordinates and seed.
/// Ensures reproducible pseudo-random ordering of cells per plate.
fn hash_cell(face: usize, x: usize, y: usize, seed: u64) -> u64 {
    let a = splitmix64(seed ^ (face as u64).wrapping_mul(0x9E37));
    let b = splitmix64(a ^ (x as u64).wrapping_mul(0xC2B2AE3D));
    splitmix64(b ^ (y as u64).wrapping_mul(0x165667B1))
}

/// Returns one pseudo-random cell coordinate for each tectonic plate.
///
/// # Arguments
/// * `face_grid_size` - The resolution of one cube face (number of cells per axis).
/// * `plate_map` - The 3D map [6][face_grid_size][face_grid_size] assigning plate IDs to each cell.
/// * `seed` - Random seed for reproducibility.
///
/// # Returns
/// A `HashMap<plate_id, (face, x, y)>` with exactly one representative
/// cell coordinate per plate.
///
/// - Runtime is O(N/STRIDE²) with N = total number of cells.
pub fn random_cell_per_plate(
    face_grid_size: usize,
    plate_map: &PlateMap,
    seed: u64,
) -> HashMap<usize, (usize, usize, usize)> {
    let mut best: HashMap<usize, (u64, usize, usize, usize)> = HashMap::new();
    for face in 0..6 {
        let mut y = 0;
        while y < face_grid_size {
            let mut x = 0;
            while x < face_grid_size {
                let pid = plate_map[face][y][x];
                let h = hash_cell(face, x, y, seed);
                match best.get_mut(&pid) {
                    Some(entry) => {
                        if h < entry.0 {
                            *entry = (h, face, x, y);
                        }
                    }
                    None => {
                        best.insert(pid, (h, face, x, y));
                    }
                }
                x += STRIDE;
            }
            y += STRIDE;
        }
    }
    best.into_iter()
        .map(|(pid, (_, f, x, y))| (pid, (f, x, y)))
        .collect()
}