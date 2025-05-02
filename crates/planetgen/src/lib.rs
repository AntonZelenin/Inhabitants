const HEIGHT_SCALE: f32 = 0.1;

pub struct CubeFace {
    pub heightmap: Vec<Vec<f32>>, // [row][col]
    pub size: usize,              // grid size per face
}

pub struct PlanetData {
    pub faces: [CubeFace; 6],     // +X, -X, +Y, -Y, +Z, -Z
    pub size: usize,
}

pub struct PlanetGenerator {
    pub size: usize,
}

impl PlanetGenerator {
    pub fn new(size: usize) -> Self {
        Self { size }
    }

    pub fn generate(&self) -> PlanetData {
        let faces = [
            Self::generate_face(self.size, "pos_x"),
            Self::generate_face(self.size, "neg_x"),
            Self::generate_face(self.size, "pos_y"),
            Self::generate_face(self.size, "neg_y"),
            Self::generate_face(self.size, "pos_z"),
            Self::generate_face(self.size, "neg_z"),
        ];

        PlanetData {
            faces,
            size: self.size,
        }
    }

    fn generate_face(size: usize, _face_name: &str) -> CubeFace {
        let mut heightmap = vec![vec![0.0; size]; size];

        for y in 0..size {
            for x in 0..size {
                let raw = (x as f32 * 0.05).sin() * (y as f32 * 0.05).cos();
                let height = raw * HEIGHT_SCALE;
                heightmap[y][x] = height;
            }
        }

        CubeFace { heightmap, size }
    }
}
