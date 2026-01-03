/// Plate boundary classification and coloring system
///
/// This module calculates the interaction type at plate boundaries by analyzing
/// the relative velocity of adjacent plates.

use crate::plate::TectonicPlate;
use crate::planet::PlateMap;
use glam::Vec3;

/// Type of plate boundary interaction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryType {
    /// Convergent boundary - plates colliding (red)
    Convergent,
    /// Divergent boundary - plates spreading apart (blue)
    Divergent,
    /// Transform boundary - plates sliding past each other (yellow)
    Transform,
}

impl BoundaryType {
    /// Get the color for this boundary type
    pub fn color(&self) -> [f32; 3] {
        match self {
            BoundaryType::Convergent => [1.0, 0.0, 0.0], // Red
            BoundaryType::Divergent => [0.0, 0.5, 1.0],  // Blue
            BoundaryType::Transform => [1.0, 1.0, 0.0],  // Yellow
        }
    }
}

/// Boundary information for the entire planet
pub struct BoundaryData {
    /// Boundary type for each cell (None = interior, Some = boundary)
    pub boundaries: [Vec<Vec<Option<BoundaryType>>>; 6],
    /// Distance from boundary (0.0 = exact boundary, 1.0+ = far from any boundary)
    /// Used for fade-out effect
    pub boundary_distances: [Vec<Vec<f32>>; 6],
}

impl BoundaryData {
    /// Calculate boundary types for all plate boundaries
    ///
    /// Creates wider boundaries (5% of planet size) with fade-out effect.
    ///
    /// # Algorithm
    /// 1. Find exact boundary cells (adjacent to different plates)
    /// 2. Classify boundary type (convergent/divergent/transform)
    /// 3. Calculate distance field from boundaries
    /// 4. Apply fade-out based on distance
    pub fn calculate(
        face_grid_size: usize,
        plate_map: &PlateMap,
        plates: &[TectonicPlate],
    ) -> Self {
        // Create plate ID to plate lookup
        let plate_lookup: std::collections::HashMap<usize, &TectonicPlate> =
            plates.iter().map(|p| (p.id, p)).collect();

        let mut boundaries = std::array::from_fn(|_| {
            vec![vec![None; face_grid_size]; face_grid_size]
        });

        let mut boundary_distances = std::array::from_fn(|_| {
            vec![vec![f32::INFINITY; face_grid_size]; face_grid_size]
        });

        // Step 1: Find exact boundary cells and classify them
        // Important: Mark BOTH sides of the boundary with the same classification
        for face_idx in 0..6 {
            for y in 0..face_grid_size {
                for x in 0..face_grid_size {
                    let current_plate = plate_map[face_idx][y][x];

                    // Check all 4 neighbors
                    for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && ny >= 0 && (nx as usize) < face_grid_size && (ny as usize) < face_grid_size {
                            let nx = nx as usize;
                            let ny = ny as usize;
                            let neighbor_plate = plate_map[face_idx][ny][nx];

                            // Found a boundary between different plates
                            if neighbor_plate != current_plate {
                                // Only process if not already classified (avoid duplicate work)
                                if boundaries[face_idx][y][x].is_none() {
                                    if let (Some(plate_a), Some(plate_b)) = (
                                        plate_lookup.get(&current_plate),
                                        plate_lookup.get(&neighbor_plate),
                                    ) {
                                        // Calculate boundary position (midpoint between cells)
                                        let u_curr = x as f32 / (face_grid_size - 1) as f32 * 2.0 - 1.0;
                                        let v_curr = y as f32 / (face_grid_size - 1) as f32 * 2.0 - 1.0;
                                        let u_neigh = nx as f32 / (face_grid_size - 1) as f32 * 2.0 - 1.0;
                                        let v_neigh = ny as f32 / (face_grid_size - 1) as f32 * 2.0 - 1.0;

                                        let pos_curr = Vec3::from(crate::generator::cube_face_point(face_idx, u_curr, v_curr)).normalize();
                                        let pos_neigh = Vec3::from(crate::generator::cube_face_point(face_idx, u_neigh, v_neigh)).normalize();
                                        let boundary_position = ((pos_curr + pos_neigh) * 0.5).normalize();

                                        let boundary_type = classify_boundary(boundary_position, plate_a, plate_b);

                                        // Mark BOTH sides with the same boundary type
                                        boundaries[face_idx][y][x] = Some(boundary_type);
                                        boundary_distances[face_idx][y][x] = 0.0;
                                        boundaries[face_idx][ny][nx] = Some(boundary_type);
                                        boundary_distances[face_idx][ny][nx] = 0.0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Step 2: Calculate distance field using flood fill
        // Boundary width: 5% of grid size (roughly 5% of planet radius)
        let boundary_width = (face_grid_size as f32 * 0.05).max(3.0) as usize;

        // Simple distance propagation
        for dist in 1..=boundary_width {
            for face_idx in 0..6 {
                for y in 0..face_grid_size {
                    for x in 0..face_grid_size {
                        if boundary_distances[face_idx][y][x] == (dist - 1) as f32 {
                            // Propagate to neighbors
                            for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                                let nx = x as i32 + dx;
                                let ny = y as i32 + dy;
                                if nx >= 0 && ny >= 0 && (nx as usize) < face_grid_size && (ny as usize) < face_grid_size {
                                    let nx = nx as usize;
                                    let ny = ny as usize;

                                    if boundary_distances[face_idx][ny][nx] == f32::INFINITY {
                                        boundary_distances[face_idx][ny][nx] = dist as f32;

                                        // Inherit boundary type from parent
                                        if boundaries[face_idx][y][x].is_some() && boundaries[face_idx][ny][nx].is_none() {
                                            boundaries[face_idx][ny][nx] = boundaries[face_idx][y][x];
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self { boundaries, boundary_distances }
    }

    /// Get the boundary type at a specific cell
    pub fn get_boundary(&self, face: usize, x: usize, y: usize) -> Option<BoundaryType> {
        self.boundaries[face][y][x]
    }

    /// Get boundary color with distance-based opacity
    /// Returns (color, opacity) where opacity is 1.0 at boundary, 0.0 far away
    pub fn get_boundary_color(&self, face: usize, x: usize, y: usize) -> Option<([f32; 3], f32)> {
        if let Some(boundary_type) = self.boundaries[face][y][x] {
            let distance = self.boundary_distances[face][y][x];

            // Calculate opacity: 1.0 at boundary (distance 0), fade to 0.0 at max distance
            // Use smooth falloff curve
            let max_dist = 10.0; // Approximately 5% of grid size
            let normalized_dist = (distance / max_dist).min(1.0);
            let opacity = 1.0 - normalized_dist * normalized_dist; // Quadratic falloff for smooth fade

            Some((boundary_type.color(), opacity))
        } else {
            None
        }
    }
}

/// Classify the type of boundary interaction between two plates
///
/// Uses the plate centers to determine a consistent classification across the entire boundary.
///
/// # Algorithm
/// 1. Find the boundary midpoint between the two plate centers
/// 2. Calculate tangent velocities of both plates at this reference point
/// 3. Compute relative velocity component perpendicular to the line connecting centers
/// 4. Classify based on whether plates are converging, diverging, or sliding
fn classify_boundary(
    position: Vec3,
    plate_a: &TectonicPlate,
    plate_b: &TectonicPlate,
) -> BoundaryType {
    // The boundary lies on the great circle perpendicular to the line connecting centers.
    // We care about the component of relative velocity *across* that boundary within the local tangent plane.
    let center_line = (plate_a.center - plate_b.center).normalize_or_zero();
    if center_line.length_squared() < 1e-6 {
        return BoundaryType::Transform;
    };

    // Project center_line into the tangent plane at the boundary position to get the across-boundary normal.
    let tangent_normal = (center_line - position * center_line.dot(position)).normalize_or_zero();
    if tangent_normal.length_squared() < 1e-8 {
        return BoundaryType::Transform;
    };

    // Tangent velocity at position: angular_velocity × position
    let vel_a = plate_a.angular_velocity.cross(position);
    let vel_b = plate_b.angular_velocity.cross(position);
    let relative_vel = vel_a - vel_b;

    let rel_speed = relative_vel.length();
    if rel_speed < 1e-6 {
        return BoundaryType::Transform;
    };

    // Positive = A moving toward B across the boundary; negative = moving away.
    let normal_component = relative_vel.dot(tangent_normal);

    // Threshold scaled to motion magnitude to avoid classifying near-zero noise as convergent/divergent.
    let threshold = 0.005_f32.max(rel_speed * 0.02);

    if normal_component > threshold {
        BoundaryType::Convergent
    } else if normal_component < -threshold {
        BoundaryType::Divergent
    } else {
        BoundaryType::Transform
    }
}
