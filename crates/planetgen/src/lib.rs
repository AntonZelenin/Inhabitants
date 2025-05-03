const CELLS_PER_UNIT: f32 = 1.0;

pub struct CubeFace {
    pub heightmap: Vec<Vec<f32>>,
    pub grid_size: usize,
}

pub struct PlanetData {
    pub faces: [CubeFace; 6],
    pub face_grid_size: usize,
    pub radius: f32,
}

pub struct PlanetGenerator {
    pub radius: f32,
    pub cells_per_unit: f32,
}

impl PlanetGenerator {
    pub fn new(radius: f32) -> Self {
        Self { radius, cells_per_unit: CELLS_PER_UNIT }
    }

    pub fn new_with_cells(radius: f32, cells_per_unit: f32) -> Self {
        Self { radius, cells_per_unit }
    }

    pub fn generate(&self) -> PlanetData {
        let grid_size = (self.radius * self.cells_per_unit).ceil() as usize + 1;

        let faces = [
            Self::generate_face(grid_size),
            Self::generate_face(grid_size),
            Self::generate_face(grid_size),
            Self::generate_face(grid_size),
            Self::generate_face(grid_size),
            Self::generate_face(grid_size),
        ];

        PlanetData {
            faces,
            face_grid_size: grid_size,
            radius: self.radius,
        }
    }

    fn generate_face(size: usize) -> CubeFace {
        let mut heightmap = vec![vec![0.0; size]; size];

        for y in 0..size {
            for x in 0..size {
                heightmap[y][x] = (x as f32 * 0.05).sin() * (y as f32 * 0.05).cos() * 0.1;
            }
        }

        CubeFace { heightmap, grid_size: size }
    }
}
