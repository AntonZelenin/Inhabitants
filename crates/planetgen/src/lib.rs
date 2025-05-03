use noise::{NoiseFn, Perlin};
use rand::{random_bool, random_range};

const CELLS_PER_UNIT: f32 = 1.0;

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn normalize(self) -> Self {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
                z: self.z / mag,
            }
        } else {
            self
        }
    }

    pub fn distance(self, other: Vec3) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

#[derive(Clone)]
pub struct CubeFace {
    pub heightmap: Vec<Vec<f32>>,
}

pub enum PlateType {
    Continental,
    Oceanic,
}

pub struct NoiseConfig {
    perlin: Perlin,
    frequency: f32,
    amplitude: f32,
}

impl NoiseConfig {
    /// Create a new noise config with explicit seed for reproducibility
    pub fn new(seed: u32, frequency: f32, amplitude: f32) -> Self {
        Self {
            perlin: Perlin::new(seed),
            frequency,
            amplitude,
        }
    }

    pub fn sample(&self, dir: Vec3) -> f32 {
        let x = dir.x * self.frequency;
        let y = dir.y * self.frequency;
        let z = dir.z * self.frequency;
        self.perlin.get([x as f64, y as f64, z as f64]) as f32 * self.amplitude
    }
}

pub struct TectonicPlate {
    pub id: usize,
    pub seed_dir: Vec3,
    pub plate_type: PlateType,
    pub noise_config: NoiseConfig,
    pub color: [f32; 4],
}

pub struct PlanetData {
    pub faces: [CubeFace; 6],
    pub face_grid_size: usize,
    pub radius: f32,
    pub plate_map: Vec<Vec<Vec<usize>>>,
    pub plates: Vec<TectonicPlate>,
}

pub struct PlanetGenerator {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
}

impl PlanetGenerator {
    pub fn new(radius: f32) -> Self {
        Self {
            radius,
            cells_per_unit: CELLS_PER_UNIT,
            num_plates: 10,
        }
    }

    pub fn new_with_cells(radius: f32, cells_per_unit: f32) -> Self {
        Self {
            radius,
            cells_per_unit,
            num_plates: 10,
        }
    }

    pub fn with_plate_count(mut self, count: usize) -> Self {
        self.num_plates = count;
        self
    }

    pub fn generate(&self) -> PlanetData {
        let grid_size = (self.radius * self.cells_per_unit).ceil() as usize + 1;
        let plates = self.generate_plates();
        let plate_map = self.assign_plates(grid_size, &plates);
        let faces = self.generate_faces(grid_size, &plates, &plate_map);

        PlanetData {
            faces,
            face_grid_size: grid_size,
            radius: self.radius,
            plate_map,
            plates,
        }
    }

    fn generate_plates(&self) -> Vec<TectonicPlate> {
        let debug_colors = [
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
        (0..self.num_plates)
            .map(|id| {
                // explicit seed for Perlin
                let noise_seed = random_range(0_u32..u32::MAX);
                // random plate direction
                let seed_dir = Vec3::new(
                    random_range(-1.0..1.0),
                    random_range(-1.0..1.0),
                    random_range(-1.0..1.0),
                )
                .normalize();

                let plate_type = if random_bool(0.5) {
                    PlateType::Continental
                } else {
                    PlateType::Oceanic
                };

                let (freq, amp) = match plate_type {
                    PlateType::Continental => (3.0, 0.7),
                    PlateType::Oceanic => (2.0, 0.05),
                };

                let noise_config = NoiseConfig::new(noise_seed, freq, amp);
                let color = debug_colors[id % debug_colors.len()];

                TectonicPlate {
                    id,
                    seed_dir,
                    plate_type,
                    noise_config,
                    color,
                }
            })
            .collect()
    }

    fn assign_plates(&self, size: usize, plates: &[TectonicPlate]) -> Vec<Vec<Vec<usize>>> {
        let mut plate_map = vec![vec![vec![0; size]; size]; 6];

        for face_idx in 0..6 {
            for y in 0..size {
                let v = (y as f32 / (size - 1) as f32) * 2.0 - 1.0;

                for x in 0..size {
                    let u = (x as f32 / (size - 1) as f32) * 2.0 - 1.0;
                    let (dx, dy, dz) = cube_face_point(face_idx, u, v);
                    let dir = Vec3::new(dx, dy, dz).normalize();

                    let plate_id = plates
                        .iter()
                        .min_by(|a, b| {
                            a.seed_dir
                                .distance(dir)
                                .partial_cmp(&b.seed_dir.distance(dir))
                                .unwrap()
                        })
                        .unwrap()
                        .id;

                    plate_map[face_idx][y][x] = plate_id;
                }
            }
        }

        plate_map
    }

    fn generate_faces(
        &self,
        size: usize,
        plates: &[TectonicPlate],
        plate_map: &Vec<Vec<Vec<usize>>>,
    ) -> [CubeFace; 6] {
        let blank = CubeFace {
            heightmap: vec![vec![0.0; size]; size],
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
            for y in 0..size {
                let v = (y as f32 / (size - 1) as f32) * 2.0 - 1.0;

                for x in 0..size {
                    let u = (x as f32 / (size - 1) as f32) * 2.0 - 1.0;
                    let (dx, dy, dz) = cube_face_point(face_idx, u, v);
                    let dir = Vec3::new(dx, dy, dz).normalize();

                    let plate_id = plate_map[face_idx][y][x];
                    let height = plates[plate_id].noise_config.sample(dir);

                    faces[face_idx].heightmap[y][x] = height;
                }
            }
        }

        faces
    }
}

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
