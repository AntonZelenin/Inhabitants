use crate::config::NoiseConfig;
use crate::constants::*;
use crate::planet::*;
use crate::plate::TectonicPlate;
use glam::Vec3;
use rand::{random_bool, random_range};
use std::collections::HashMap;

/// Spatial frequency of the flow field used to bend plate boundaries.
/// Lower values produce larger, smoother swirls; higher values add finer detail.
/// Examples: 0.15–0.40 = broad, plate-scale bends; 0.5–1.0 = medium; >1.5 = busy/jittery.
pub const FLOW_WARP_FREQ: f32 = 0.45;
/// Strength of the flow vectors before projection to the tangent plane.
/// Controls how hard each step pushes. Examples: 0.10 subtle, 0.25 balanced (default), 0.40 strong, >0.60 chaotic.
pub const FLOW_WARP_AMP: f32 = 0.25;
/// Number of advection steps applied per cell. More steps = more coherent, larger-scale displacement (but slower).
/// Examples: 1 minimal, 2–4 typical, 5–8 heavy warp.
pub const FLOW_WARP_STEPS: usize = 3;
/// Angular step size per advection step (radians). Sets the along-surface distance moved each step.
/// Examples: 0.05 (~3°) subtle, 0.12 (~7°) default, 0.25 (~14°) strong, >0.50 (~29°) extreme.
pub const FLOW_WARP_STEP_ANGLE: f32 = 0.12;

pub struct PlanetGenerator {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
}

impl PlanetGenerator {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            cells_per_unit: CELLS_PER_UNIT,
            // default values, will be replaced by planet settings
            num_plates: 0,
            num_micro_plates: 0,
        }
    }

    pub fn generate(&self) -> PlanetData {
        // Each cube face represents a square section of the unit sphere, scaled the planet’s radius.
        // cells_per_unit = how many grid cells per 1 unit of world space
        // radius * cells_per_unit = number of cells from edge to edge on one face
        // +1 = adds 1 to include both start and end of the grid (for vertices, not just quads)
        let face_grid_size = (self.radius * self.cells_per_unit).ceil() as usize + 1;

        let mut plates = self.generate_plates();
        let mut plate_map = self.assign_plates(face_grid_size, &plates);

        let micros = self.generate_microplates(face_grid_size, &plates, &plate_map);
        plates.extend(micros);

        plate_map = self.assign_plates(face_grid_size, &plates);
        majority_smooth(face_grid_size, &mut plate_map);

        let faces = self.generate_faces(face_grid_size, &plates, &plate_map);
        PlanetData {
            faces,
            face_grid_size,
            radius: self.radius,
            plate_map,
            plates,
        }
    }

    fn make_plate(
        &self,
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
    fn generate_plates(&self) -> Vec<TectonicPlate> {
        let mut directions: Vec<Vec3> = (0..self.num_plates)
            .map(|_| {
                Vec3::new(
                    random_range(-1.0..1.0),
                    random_range(-1.0..1.0),
                    random_range(-1.0..1.0),
                )
                .normalize()
            })
            .collect();

        self.enforce_minimum_plate_distance(&mut directions);

        directions
            .into_iter()
            .enumerate()
            .map(|(id, direction)| {
                let plate_type = if random_bool(CONTINENTAL_PLATE_PROBABILITY) {
                    PlateType::Continental
                } else {
                    PlateType::Oceanic
                };
                let (freq, amp) = match plate_type {
                    PlateType::Continental => (CONTINENTAL_FREQ, CONTINENTAL_AMP),
                    PlateType::Oceanic => (OCEANIC_FREQ, OCEANIC_AMP),
                };
                self.make_plate(
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
    fn enforce_minimum_plate_distance(&self, directions: &mut Vec<Vec3>) {
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
                            let distance_deficit =
                                MIN_PLATE_SEPARATION_CHORD_DISTANCE - chord_distance;
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

    fn advect_dir(&self, p: Vec3, nx: &NoiseConfig, ny: &NoiseConfig, nz: &NoiseConfig) -> Vec3 {
        let mut d = p;
        for _ in 0..FLOW_WARP_STEPS {
            let v = Vec3::new(nx.sample(d), ny.sample(d), nz.sample(d));
            let t = v - d * d.dot(v);
            let tl = t.length();
            if tl > 1e-6 {
                let tn = t / tl;
                let c = FLOW_WARP_STEP_ANGLE.cos();
                let s = FLOW_WARP_STEP_ANGLE.sin();
                d = (d * c + tn * s).normalize();
            } else {
                break;
            }
        }
        d
    }

    /// Generates smaller microplates along the boundaries of major plates
    ///
    /// Microplates are placed at locations where different major plates meet,
    /// creating more detailed terrain features along plate boundaries.
    fn generate_microplates(
        &self,
        face_grid_size: usize,
        plates: &[TectonicPlate],
        plate_map: &PlateMap,
    ) -> Vec<TectonicPlate> {
        (0..self.num_micro_plates)
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
                    random_range(MICRO_PLATE_JITTER_RANGE),
                    random_range(MICRO_PLATE_JITTER_RANGE),
                    random_range(MICRO_PLATE_JITTER_RANGE),
                );
                let seed_dir = (base_dir + jitter).normalize();
                // smaller scale noise
                let freq = CONTINENTAL_FREQ * MICRO_PLATE_FREQUENCY_MULTIPLIER;
                let amp = CONTINENTAL_AMP * MICRO_PLATE_AMPLITUDE_MULTIPLIER;
                self.make_plate(
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
    /// - compare this grid cell's direction with ALL tectonic plates' direction vectors (a small
    ///   distortion is applied to the grid cell direction to make plate boundaries less square);
    /// - the plate whose direction is closest (smallest angular distance) "wins" that grid cell
    /// - store the winner: Put that winning plate's ID into map[face][y][x]
    fn assign_plates(&self, face_grid_size: usize, plates: &[TectonicPlate]) -> PlateMap {
        let mut map = vec![vec![vec![0; face_grid_size]; face_grid_size]; 6];

        // Precompute plate vectors
        let pre: Vec<(Vec3, f32, usize)> = plates
            .iter()
            .map(|p| {
                let w = match p.size_class {
                    PlateSizeClass::Regular => 1.0,
                    PlateSizeClass::Micro => MICRO_PLATE_WEIGHT_FACTOR,
                };
                (p.direction.normalize(), w * w, p.id)
            })
            .collect();

        let warp_x = NoiseConfig::new(
            random_range(0_u32..u32::MAX),
            PLATE_BOUNDARY_DISTORTION_FREQUENCY,
            PLATE_BOUNDARY_DISTORTION_AMPLITUDE,
        );
        let warp_y = NoiseConfig::new(
            random_range(0_u32..u32::MAX),
            PLATE_BOUNDARY_DISTORTION_FREQUENCY,
            PLATE_BOUNDARY_DISTORTION_AMPLITUDE,
        );
        let warp_z = NoiseConfig::new(
            random_range(0_u32..u32::MAX),
            PLATE_BOUNDARY_DISTORTION_FREQUENCY,
            PLATE_BOUNDARY_DISTORTION_AMPLITUDE,
        );
        let flow_x = NoiseConfig::new(random_range(0_u32..u32::MAX), FLOW_WARP_FREQ, FLOW_WARP_AMP);
        let flow_y = NoiseConfig::new(random_range(0_u32..u32::MAX), FLOW_WARP_FREQ, FLOW_WARP_AMP);
        let flow_z = NoiseConfig::new(random_range(0_u32..u32::MAX), FLOW_WARP_FREQ, FLOW_WARP_AMP);

        let inv = 1.0 / (face_grid_size as f32 - 1.0);
        for f in 0..6 {
            for y in 0..face_grid_size {
                let v = y as f32 * inv * 2.0 - 1.0;
                for x in 0..face_grid_size {
                    let u = x as f32 * inv * 2.0 - 1.0;
                    let mut dir = Vec3::from(cube_face_point(f, u, v)).normalize();
                    let r = Vec3::new(warp_x.sample(dir), warp_y.sample(dir), warp_z.sample(dir));
                    let t = r - dir * dir.dot(r);
                    dir = (dir + t * PLATE_BOUNDARY_WARP_MULTIPLIER).normalize();
                    let dir = self.advect_dir(dir, &flow_x, &flow_y, &flow_z);

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
        &self,
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

/// Smooths thin, noisy seams in the plate map using a single-pass majority vote.
/// For each cell, counts its 8 neighbours plus itself (self counts double) and
/// assigns the most frequent plate ID to the cell.
///
/// This removes 1–2 cell cracks while preserving large-scale shapes.
/// Call once after the final `assign_plates`.
///
/// # Behaviour
/// Neighbourhood: 8-connectivity. Self weight: 2.
/// Ties: winner is the first plate ID reaching the max count during scan (stable).
///
/// # Complexity
/// O(6 · face_n²) time, O(face_n²) extra per-face buffer.
///
/// # Notes
/// If over-smoothing occurs, reduce self weight or drop the diagonal neighbours.
fn majority_smooth(face_n: usize, map: &mut PlateMap) {
    for f in 0..6 {
        let mut out = map[f].clone();
        for y in 0..face_n {
            for x in 0..face_n {
                let mut hist: HashMap<usize, u32> = HashMap::new();
                let pid = map[f][y][x];
                *hist.entry(pid).or_insert(0) += 2;
                for (dx, dy) in [
                    (-1i32, 0i32),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                ] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && (nx as usize) < face_n && (ny as usize) < face_n {
                        let q = map[f][ny as usize][nx as usize];
                        *hist.entry(q).or_insert(0) += 1;
                    }
                }
                let mut best = pid;
                let mut best_v = 0u32;
                for (k, v) in hist {
                    if v > best_v {
                        best_v = v;
                        best = k;
                    }
                }
                out[y][x] = best;
            }
        }
        map[f] = out;
    }
}
