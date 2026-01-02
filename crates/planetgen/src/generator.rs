use crate::config::{NoiseConfig, PlanetGenConfig};
use crate::constants::*;
use crate::planet::*;
use crate::plate::TectonicPlate;
use crate::tools::splitmix64;
use glam::Vec3;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};

pub struct PlanetGenerator {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
    pub seed: u64,
    pub flow_warp_freq: f32,
    pub flow_warp_amp: f32,
    pub flow_warp_steps: usize,
    pub flow_warp_step_angle: f32,
    config: PlanetGenConfig,
}

impl PlanetGenerator {
    pub fn new(radius: f32) -> Self {
        let config = crate::get_config();
        Self {
            radius,
            cells_per_unit: config.generation.cells_per_unit,
            // default values, will be replaced by planet settings
            num_plates: config.generation.default_num_plates,
            num_micro_plates: config.generation.default_num_micro_plates,
            seed: 0,
            flow_warp_freq: config.flow_warp.default_freq,
            flow_warp_amp: config.flow_warp.default_amp,
            flow_warp_steps: config.flow_warp.default_steps,
            flow_warp_step_angle: config.flow_warp.default_step_angle,
            config,
        }
    }

    /// Apply custom continent generation parameters
    pub fn with_continent_config(&mut self, continent_config: crate::config::ContinentConfig) {
        self.config.continents = continent_config;
    }

    // --- Deterministic RNG helpers (domain-separated) ---
    fn fnv1a64(mut acc: u64, bytes: &[u8]) -> u64 {
        const FNV_OFFSET: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;
        if acc == 0 {
            acc = FNV_OFFSET;
        }
        let mut h = acc;
        for &b in bytes {
            h ^= b as u64;
            h = h.wrapping_mul(FNV_PRIME);
        }
        h
    }

    fn seed32_for(&self, domain: &str) -> [u8; 32] {
        // Mix master seed with domain label via FNV1a64, then expand with SplitMix64
        let s = Self::fnv1a64(self.seed, domain.as_bytes());
        let mut out = [0u8; 32];
        for i in 0..4 {
            let v = splitmix64(s ^ (i as u64));
            out[i * 8..(i + 1) * 8].copy_from_slice(&v.to_le_bytes());
        }
        out
    }

    fn seed_u32_for(&self, domain: &str) -> u32 {
        // Take lower 32 bits of SplitMix64 expansion for quick u32 seeds
        let v = splitmix64(Self::fnv1a64(self.seed, domain.as_bytes()));
        (v & 0xFFFF_FFFF) as u32
    }

    fn rng_for_indexed(&self, domain: &str, idx: u64) -> StdRng {
        let key = format!("{domain}/{idx}");
        StdRng::from_seed(self.seed32_for(&key))
    }

    pub fn generate(&self) -> PlanetData {
        // Each cube face represents a square section of the unit sphere, scaled the planet's radius.
        // cells_per_unit = how many grid cells per 1 unit of world space
        // radius * cells_per_unit = number of cells from edge to edge on one face
        // +1 = adds 1 to include both start and end of the grid (for vertices, not just quads)
        let face_grid_size = (self.radius * self.cells_per_unit).ceil() as usize + 1;

        let mut plates = self.generate_plates();
        let mut plate_map = self.assign_plates(face_grid_size, &plates);

        let micros = self.generate_microplates(face_grid_size, &plates, &plate_map);
        plates.extend(micros);

        plate_map = self.assign_plates(face_grid_size, &plates);

        // Apply plate merging (always enabled with probabilistic selection)
        self.merge_plates(face_grid_size, &mut plate_map);

        majority_smooth(face_grid_size, &mut plate_map);

        // Create continent noise configuration using custom config (independent of plates)
        let continent_seed = self.seed_u32_for("continents");
        let continent_noise = crate::continents::ContinentNoiseConfig::from_config(
            continent_seed, 
            &self.config.continents
        );

        let faces = self.generate_faces(face_grid_size, &plates, &plate_map, &continent_noise);
        PlanetData {
            faces,
            face_grid_size,
            radius: self.radius,
            plate_map,
            plates,
            continent_noise,
        }
    }

    fn make_plate(
        &self,
        id: usize,
        direction: Vec3,
        size_class: PlateSizeClass,
        freq: f32,
        amp: f32,
        noise_seed: u32,
    ) -> TectonicPlate {
        let color = DEBUG_COLORS[id % DEBUG_COLORS.len()];
        TectonicPlate {
            id,
            direction,
            size_class,
            noise_config: NoiseConfig::new(noise_seed, freq, amp),
            debug_color: color,
        }
    }

    /// Generates the main tectonic plates for the planet
    ///
    /// Creates random plates with consistent noise parameters for plate tectonics simulation.
    /// Each plate gets a random seed direction on the unit sphere.
    /// Note: Continents are generated separately and are not tied to plate types.
    fn generate_plates(&self) -> Vec<TectonicPlate> {
        // Derive a separate RNG per-plate for directions
        let mut directions: Vec<Vec3> = (0..self.num_plates)
            .map(|i| {
                let mut rng = self.rng_for_indexed("plates/direction", i as u64);
                Vec3::new(
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                    rng.random_range(-1.0..1.0),
                )
                .normalize()
            })
            .collect();

        self.enforce_minimum_plate_distance(&mut directions);

        directions
            .into_iter()
            .enumerate()
            .map(|(id, direction)| {
                // Use consistent noise parameters for all plates (tectonic movement only)
                let freq = self.config.generation.continental_freq;
                // todo does * 0.3 make sense here?
                let amp = self.config.generation.continental_amp * 0.3; // Reduced for base terrain
                let noise_seed = self.seed_u32_for(&format!("plates/noise/{id}"));
                self.make_plate(
                    id,
                    direction,
                    PlateSizeClass::Regular,
                    freq,
                    amp,
                    noise_seed,
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
                    if chord_distance < self.config.plates.min_separation_chord_distance {
                        any_moved = true;

                        // Calculate the vector between the two points
                        let diff = dir_j - dir_i;
                        let diff_length = diff.length();

                        if diff_length > eps {
                            let distance_deficit =
                                self.config.plates.min_separation_chord_distance - chord_distance;
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
        for _ in 0..self.flow_warp_steps {
            let v = Vec3::new(nx.sample(d), ny.sample(d), nz.sample(d));
            let t = v - d * d.dot(v);
            let tl = t.length();
            if tl > 1e-6 {
                let tn = t / tl;
                let c = self.flow_warp_step_angle.cos();
                let s = self.flow_warp_step_angle.sin();
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

                // RNG dedicated for placement sampling, stable per microplate index
                let mut rng_pick = self.rng_for_indexed("microplates/pick", i as u64);

                let (f, x, y) = loop {
                    let f: usize = rng_pick.random_range(0..6);
                    let y: usize = rng_pick.random_range(0..face_grid_size);
                    let x: usize = rng_pick.random_range(0..face_grid_size);
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
                // tiny jitter so seed stays close to boundary; independent RNG for jitter per microplate
                let mut rng_jitter = self.rng_for_indexed("microplates/jitter", i as u64);
                let jitter = Vec3::new(
                    rng_jitter.random_range(self.config.microplate_jitter_range()),
                    rng_jitter.random_range(self.config.microplate_jitter_range()),
                    rng_jitter.random_range(self.config.microplate_jitter_range()),
                );
                let seed_dir = (base_dir + jitter).normalize();
                // smaller scale noise
                let freq = self.config.generation.continental_freq * self.config.microplates.frequency_multiplier;
                let amp = self.config.generation.continental_amp * self.config.microplates.amplitude_multiplier * 0.3;
                let noise_seed = self.seed_u32_for(&format!("microplates/noise/{id}"));
                self.make_plate(
                    id,
                    seed_dir,
                    PlateSizeClass::Micro,
                    freq,
                    amp,
                    noise_seed,
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
                    PlateSizeClass::Micro => self.config.plates.micro_plate_weight_factor,
                };
                (p.direction.normalize(), w * w, p.id)
            })
            .collect();

        // Deterministic warp and flow noise seeds per axis
        let warp_x = NoiseConfig::new(
            self.seed_u32_for("assign_plates/warp/x"),
            self.config.boundaries.distortion_frequency,
            self.config.boundaries.distortion_amplitude,
        );
        let warp_y = NoiseConfig::new(
            self.seed_u32_for("assign_plates/warp/y"),
            self.config.boundaries.distortion_frequency,
            self.config.boundaries.distortion_amplitude,
        );
        let warp_z = NoiseConfig::new(
            self.seed_u32_for("assign_plates/warp/z"),
            self.config.boundaries.distortion_frequency,
            self.config.boundaries.distortion_amplitude,
        );
        let flow_x = NoiseConfig::new(
            self.seed_u32_for("assign_plates/flow/x"),
            self.flow_warp_freq,
            self.flow_warp_amp,
        );
        let flow_y = NoiseConfig::new(
            self.seed_u32_for("assign_plates/flow/y"),
            self.flow_warp_freq,
            self.flow_warp_amp,
        );
        let flow_z = NoiseConfig::new(
            self.seed_u32_for("assign_plates/flow/z"),
            self.flow_warp_freq,
            self.flow_warp_amp,
        );

        let inv = 1.0 / (face_grid_size as f32 - 1.0);
        for f in 0..6 {
            for y in 0..face_grid_size {
                let v = y as f32 * inv * 2.0 - 1.0;
                for x in 0..face_grid_size {
                    let u = x as f32 * inv * 2.0 - 1.0;
                    let mut dir = Vec3::from(cube_face_point(f, u, v)).normalize();
                    let r = Vec3::new(warp_x.sample(dir), warp_y.sample(dir), warp_z.sample(dir));
                    let t = r - dir * dir.dot(r);
                    dir = (dir + t * self.config.boundaries.warp_multiplier).normalize();
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
    /// Uses multi-octave continent noise to create realistic terrain with continents and oceans.
    /// Plate data is kept for future tectonic simulation but doesn't directly affect height.
    fn generate_faces(
        &self,
        face_grid_size: usize,
        plates: &[TectonicPlate],
        plate_map: &PlateMap,
        continent_noise: &crate::continents::ContinentNoiseConfig,
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
                    
                    // Generate height using continent noise (independent of plates)
                    let height = continent_noise.sample_height(dir);
                    
                    // Optional: Add subtle plate-based variation for future tectonic features
                    let plate_id = plate_map[face_idx][y][x];
                    let plate_variation = plates[plate_id].noise_config.sample(dir) * 0.1;
                    
                    faces[face_idx].heightmap[y][x] = height + plate_variation;
                }
            }
        }
        faces
    }

    /// Merges randomly selected plates with their neighbors using probabilistic selection
    ///
    /// Uses deterministic probabilities based on the master seed:
    /// - 10% chance for each plate to be selected as a primary for merging
    /// - 30% chance to select 2 neighbors, otherwise 1 neighbor
    fn merge_plates(&self, face_grid_size: usize, plate_map: &mut PlateMap) {
        // Build adjacency map and count plate areas
        let adjacency = self.build_plate_adjacency(face_grid_size, plate_map);
        let plate_areas = self.count_plate_areas(face_grid_size, plate_map);

        // Select plates for merging using probabilistic selection
        let merge_map = self.select_plates_for_merging_probabilistic(&adjacency, &plate_areas);

        // Apply the merges by remapping plate IDs in the plate_map
        self.apply_plate_merges(face_grid_size, plate_map, &merge_map);
    }

    /// Builds adjacency relationships between plates by scanning the plate_map
    fn build_plate_adjacency(
        &self,
        face_grid_size: usize,
        plate_map: &PlateMap,
    ) -> HashMap<usize, HashSet<usize>> {
        use std::collections::HashSet;

        let mut adjacency: HashMap<usize, HashSet<usize>> = HashMap::new();

        for face_idx in 0..6 {
            for y in 0..face_grid_size {
                for x in 0..face_grid_size {
                    let current_plate = plate_map[face_idx][y][x];

                    // Check right neighbor
                    if x + 1 < face_grid_size {
                        let right_plate = plate_map[face_idx][y][x + 1];
                        if right_plate != current_plate {
                            adjacency.entry(current_plate).or_insert_with(HashSet::new).insert(right_plate);
                            adjacency.entry(right_plate).or_insert_with(HashSet::new).insert(current_plate);
                        }
                    }

                    // Check down neighbor
                    if y + 1 < face_grid_size {
                        let down_plate = plate_map[face_idx][y + 1][x];
                        if down_plate != current_plate {
                            adjacency.entry(current_plate).or_insert_with(HashSet::new).insert(down_plate);
                            adjacency.entry(down_plate).or_insert_with(HashSet::new).insert(current_plate);
                        }
                    }
                }
            }
        }

        adjacency
    }

    /// Counts the number of cells (area) for each plate
    fn count_plate_areas(&self, face_grid_size: usize, plate_map: &PlateMap) -> HashMap<usize, usize> {
        let mut areas: HashMap<usize, usize> = HashMap::new();

        for face_idx in 0..6 {
            for y in 0..face_grid_size {
                for x in 0..face_grid_size {
                    let plate_id = plate_map[face_idx][y][x];
                    *areas.entry(plate_id).or_insert(0) += 1;
                }
            }
        }

        areas
    }

    /// Selects plates for merging using probabilistic selection based on master seed
    fn select_plates_for_merging_probabilistic(
        &self,
        adjacency: &HashMap<usize, HashSet<usize>>,
        plate_areas: &HashMap<usize, usize>,
    ) -> HashMap<usize, usize> {
        use std::collections::HashSet;

        let mut merge_map: HashMap<usize, usize> = HashMap::new();
        let mut used_plates: HashSet<usize> = HashSet::new();

        // Get all plates with neighbors, sorted by area (largest first) for deterministic order
        let mut candidates: Vec<(usize, usize)> = adjacency
            .iter()
            .filter(|(_, neighbors)| !neighbors.is_empty())
            .map(|(plate_id, _)| (*plate_id, *plate_areas.get(plate_id).unwrap_or(&0)))
            .collect();
        candidates.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Use master seed-based RNG for plate selection
        let mut selection_rng = StdRng::from_seed(self.seed32_for("merge/selection"));

        for (candidate_plate, _) in candidates {
            // Skip if this plate is already involved in a merge
            if used_plates.contains(&candidate_plate) {
                continue;
            }

            // 10% chance to select this plate as a primary for merging
            if selection_rng.random::<f64>() > self.config.merging.selection_probability {
                continue;
            }

            // Get available neighbors (not already used)
            let available_neighbors: Vec<usize> = adjacency[&candidate_plate]
                .iter()
                .filter(|neighbor_id| !used_plates.contains(neighbor_id))
                .copied()
                .collect();

            if available_neighbors.is_empty() {
                continue;
            }

            // Determine number of neighbors to merge: 30% chance for 2, otherwise 1
            let max_neighbors = if selection_rng.random::<f64>() < self.config.merging.two_neighbors_probability {
                2
            } else {
                1
            };

            let num_neighbors = available_neighbors.len().min(max_neighbors);
            let mut neighbors_to_merge = available_neighbors;

            // Shuffle for random selection using the same RNG
            for i in (1..neighbors_to_merge.len()).rev() {
                let j = selection_rng.random_range(0..=i);
                neighbors_to_merge.swap(i, j);
            }

            // Take the first num_neighbors
            neighbors_to_merge.truncate(num_neighbors);

            // Mark primary and neighbors as used
            used_plates.insert(candidate_plate);
            for neighbor_id in &neighbors_to_merge {
                used_plates.insert(*neighbor_id);
                merge_map.insert(*neighbor_id, candidate_plate);
            }
        }

        merge_map
    }

    /// Applies plate merges by remapping plate IDs in the plate_map
    fn apply_plate_merges(
        &self,
        face_grid_size: usize,
        plate_map: &mut PlateMap,
        merge_map: &HashMap<usize, usize>,
    ) {
        for face_idx in 0..6 {
            for y in 0..face_grid_size {
                for x in 0..face_grid_size {
                    let current_plate = plate_map[face_idx][y][x];
                    if let Some(&primary_plate) = merge_map.get(&current_plate) {
                        plate_map[face_idx][y][x] = primary_plate;
                    }
                }
            }
        }
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
