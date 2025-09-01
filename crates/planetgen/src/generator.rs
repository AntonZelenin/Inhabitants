use crate::config::NoiseConfig;
use crate::constants::*;
use crate::planet::*;
use crate::plate::TectonicPlate;
use glam::Vec3;
use rand::{random_bool, random_range};

pub const MIN_PLATE_ANGULAR_DISTANCE: f32 = 0.3;

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
            num_plates: Self::get_number_of_plates(),
            num_micro_plates: Self::get_number_of_microplates(),
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

        let faces = self.generate_faces(face_grid_size, &plates, &plate_map);
        PlanetData {
            faces,
            face_grid_size,
            radius: self.radius,
            plate_map,
            plates,
        }
    }

    fn get_number_of_plates() -> usize {
        random_range(4..9)
    }

    fn get_number_of_microplates() -> usize {
        random_range(5..10)
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
    /// Each plate gets a random seed direction on the unit sphere, ensuring minimum
    /// distance between plate centers to prevent thin, elongated plates.
    fn generate_plates(&self) -> Vec<TectonicPlate> {
        // Generate initial random positions
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

        // Apply minimum distance constraint iteratively
        self.enforce_minimum_plate_distance(&mut directions);

        // Create plates with the adjusted positions
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

    /// Iteratively moves tectonic plate centres away from each other on the unit sphere
    /// to avoid elongated thin plates.
    ///
    /// Each plate centre is a unit `Vec3` pointing from the planet’s centre to the surface.
    /// The routine enforces a minimum angular separation between all centres by repeatedly:
    /// - scanning all unordered pairs of centres;
    /// - identifying pairs whose angular distance is below the threshold `θ_min`;
    /// - pushing each member of a pair away from the other along its
    ///   local tangent (great-circle) direction;
    /// - re-normalising all centres back onto the unit sphere;
    /// - stopping early if no centre moves in an iteration or when the iteration cap is reached.
    ///
    /// Angular proximity is tested via the dot product: “too close” ⇔ `a·b > cos(θ_min)`.
    /// The threshold is controlled by the constant `MIN_PLATE_SEPARATION_ANGLE` (radians).
    ///
    /// This reduces long, stringy plates by discouraging clustered centres
    /// without forcing perfect uniformity.
    ///
    /// # Complexity
    /// `O(P² · I)`, where `P` is the number of plates and `I` is the number of iterations.
    ///
    /// # Notes
    /// - Inputs should be unit vectors; the function renormalises after each relaxation step.
    /// - Convergence depends on the threshold and internal damping; if oscillations appear,
    /// lower the threshold slightly or increase the iteration cap.
    fn enforce_minimum_plate_distance(&self, directions: &mut Vec<Vec3>) {
        let theta_min = MIN_PLATE_ANGULAR_DISTANCE;
        let cos_min = theta_min.cos();
        let eta = 0.5;
        let eps = 1e-6_f32;
        let max_iterations = 50;

        for _ in 0..max_iterations {
            let mut moved = false;
            let mut adjustments = vec![Vec3::ZERO; directions.len()];

            for i in 0..directions.len() {
                for j in (i + 1)..directions.len() {
                    let a = directions[i];
                    let b = directions[j];
                    let dot = a.dot(b).clamp(-1.0, 1.0);
                    if dot > cos_min {
                        moved = true;
                        let ti_raw = b - a * dot;
                        let tj_raw = a - b * dot;
                        let ni = ti_raw.length();
                        let nj = tj_raw.length();
                        if ni > eps && nj > eps {
                            let delta = (cos_min - dot) * eta;
                            let ti = ti_raw / ni;
                            let tj = tj_raw / nj;
                            adjustments[i] -= ti * delta;
                            adjustments[j] -= tj * delta;
                        }
                    }
                }
            }

            for i in 0..directions.len() {
                let v = directions[i] + adjustments[i];
                if v.length_squared() > 0.0 {
                    directions[i] = v.normalize();
                }
            }

            if !moved {
                break;
            }
        }
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
                    random_range(-0.1..0.1),
                    random_range(-0.1..0.1),
                    random_range(-0.1..0.1),
                );
                let seed_dir = (base_dir + jitter).normalize();
                // smaller scale noise
                let freq = CONTINENTAL_FREQ * 1.5;
                let amp = CONTINENTAL_AMP * 0.3;
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
    /// - compare this grid cell's direction with ALL tectonic plates' direction vectors.
    /// - the plate whose direction is closest (smallest angular distance) "wins" that grid cell
    /// - store the winner: Put that winning plate's ID into map[face][y][x]
    fn assign_plates(&self, face_grid_size: usize, plates: &[TectonicPlate]) -> PlateMap {
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