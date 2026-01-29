/// Wind field generation for cube-sphere planets
/// 
/// This module generates a constant wind field across the entire planet surface.
/// The wind is represented as 2D tangent-space vectors on each cube face.

/// Represents a 2D wind vector in local tangent space
/// Convention: x = east/west, y = north/south
/// West = (-1, 0), East = (1, 0), North = (0, 1), South = (0, -1)
#[derive(Clone, Copy, Debug)]
pub struct WindVector {
    pub x: f32,  // East/West component
    pub y: f32,  // North/South component
}

impl WindVector {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Constant westward wind
    pub const fn west(speed: f32) -> Self {
        Self { x: -speed, y: 0.0 }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// A wind field for a single cube face
#[derive(Clone)]
pub struct WindFace {
    /// Wind vectors stored in row-major order
    /// Same grid structure as heightmap
    pub vectors: Vec<Vec<WindVector>>,
}

impl WindFace {
    /// Create a new wind face with constant wind direction
    pub fn new_constant(grid_size: usize, wind: WindVector) -> Self {
        let vectors = vec![vec![wind; grid_size]; grid_size];
        Self { vectors }
    }

    /// Create a wind face with all zero vectors
    pub fn new_zero(grid_size: usize) -> Self {
        Self::new_constant(grid_size, WindVector::zero())
    }
}

/// Generate a constant westward wind field for all 6 cube faces
/// 
/// # Arguments
/// * `grid_size` - The resolution of the wind grid (same as heightmap)
/// * `speed` - The constant wind speed (default: 1.0)
/// 
/// # Returns
/// An array of 6 WindFace objects, one for each cube face
pub fn generate_constant_wind_field(grid_size: usize, speed: f32) -> [WindFace; 6] {
    let westward = WindVector::west(speed);
    
    [
        WindFace::new_constant(grid_size, westward), // Face 0
        WindFace::new_constant(grid_size, westward), // Face 1
        WindFace::new_constant(grid_size, westward), // Face 2
        WindFace::new_constant(grid_size, westward), // Face 3
        WindFace::new_constant(grid_size, westward), // Face 4
        WindFace::new_constant(grid_size, westward), // Face 5
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_vector_west() {
        let wind = WindVector::west(1.0);
        assert_eq!(wind.x, -1.0);
        assert_eq!(wind.y, 0.0);
    }

    #[test]
    fn test_generate_constant_wind_field() {
        let grid_size = 10;
        let speed = 2.5;
        let wind_field = generate_constant_wind_field(grid_size, speed);
        
        // Check all 6 faces
        for face in &wind_field {
            assert_eq!(face.vectors.len(), grid_size);
            for row in &face.vectors {
                assert_eq!(row.len(), grid_size);
                for &vector in row {
                    assert_eq!(vector.x, -speed);
                    assert_eq!(vector.y, 0.0);
                }
            }
        }
    }
}
