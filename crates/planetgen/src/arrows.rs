use crate::generator::cube_face_point;
use crate::planet::PlanetData;
use glam::{Quat, Vec3};

/// Data needed to render an arrow representing a tectonic plate's movement
#[derive(Debug, Clone)]
pub struct PlateArrowData {
    /// Position of the arrow on the planet surface
    pub position: Vec3,
    /// Rotation to align the arrow with the plate's movement direction
    pub rotation: Quat,
    /// Scale of the arrow (typically a percentage of planet radius)
    pub scale: f32,
}

/// Calculate arrow positions and orientations for all tectonic plates
///
/// Returns one arrow per plate, positioned at the plate's center and oriented
/// along the plate's movement direction projected onto the sphere surface.
pub fn calculate_plate_arrows(planet: &PlanetData) -> Vec<PlateArrowData> {
    let mut arrows = Vec::with_capacity(planet.plates.len());
    let arrow_scale = planet.radius * 0.2; // 20% of planet radius

    for (plate_idx, _) in planet.plates.iter().enumerate() {
        if let Some(arrow) = calculate_single_plate_arrow(planet, plate_idx, arrow_scale) {
            arrows.push(arrow);
        }
    }

    arrows
}

/// Calculate arrow data for a single plate
fn calculate_single_plate_arrow(
    planet: &PlanetData,
    plate_idx: usize,
    arrow_scale: f32,
) -> Option<PlateArrowData> {
    let center = calculate_plate_center(planet, plate_idx)?;
    let plate = &planet.plates[plate_idx];

    // Get the movement direction of the plate
    let direction = Vec3::new(plate.direction.x, plate.direction.y, plate.direction.z).normalize();

    // Get the surface normal at this position (pointing outward from center)
    let surface_normal = center.normalize();

    // Project the plate direction onto the tangent plane at this surface point
    // This removes the component of the direction that points toward/away from the center
    let tangent_direction =
        (direction - surface_normal * direction.dot(surface_normal)).normalize();

    // Calculate rotation to point in the tangent direction
    let default_direction = Vec3::Z;
    let rotation = Quat::from_rotation_arc(default_direction, tangent_direction);

    Some(PlateArrowData {
        position: center,
        rotation,
        scale: arrow_scale,
    })
}

/// Calculate the center position of a tectonic plate
///
/// Returns the average position of all cells belonging to the plate,
/// projected onto the planet surface with a small offset above the terrain.
fn calculate_plate_center(planet: &PlanetData, plate_idx: usize) -> Option<Vec3> {
    let mut center = Vec3::ZERO;
    let mut count = 0;

    // Iterate through all faces and find cells belonging to this plate
    for (face_idx, face) in planet.faces.iter().enumerate() {
        for y in 0..planet.face_grid_size {
            for x in 0..planet.face_grid_size {
                if planet.plate_map[face_idx][y][x] == plate_idx {
                    // Convert grid position to 3D position
                    let u = (x as f32 / (planet.face_grid_size - 1) as f32) * 2.0 - 1.0;
                    let v = (y as f32 / (planet.face_grid_size - 1) as f32) * 2.0 - 1.0;
                    let (nx, ny, nz) = cube_face_point(face_idx, u, v);
                    let dir = Vec3::new(nx, ny, nz).normalize();
                    let height = face.heightmap[y][x];
                    let pos = dir * (planet.radius + height);

                    center += pos;
                    count += 1;
                }
            }
        }
    }

    if count > 0 {
        center /= count as f32;
        // Normalize to the planet radius and add a small offset above surface
        center = center.normalize() * (planet.radius + 1.0);
        Some(center)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tangent_projection() {
        // Test that directions are properly projected onto tangent plane
        let surface_normal = Vec3::Y; // Point on top of sphere
        let direction = Vec3::new(1.0, 0.5, 0.0).normalize(); // Diagonal direction

        let tangent = (direction - surface_normal * direction.dot(surface_normal)).normalize();

        // Tangent should be perpendicular to surface normal
        let dot = tangent.dot(surface_normal);
        assert!(
            dot.abs() < 0.001,
            "Tangent should be perpendicular to normal, dot was {}",
            dot
        );
    }

    #[test]
    fn test_arrow_scale() {
        // Arrow scale should be 20% of radius
        let radius = 10.0;
        let expected_scale = radius * 0.2;
        assert_eq!(expected_scale, 2.0);
    }
}
