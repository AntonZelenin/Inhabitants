// Wind direction and speed calculation module

use bevy::prelude::*;

/// Calculate wind direction and speed at a given position on the sphere
pub struct WindField;

impl WindField {
    /// Get wind velocity (direction and speed) at a position on the sphere
    /// 
    /// # Arguments
    /// * `position` - Position on the sphere surface (normalized direction vector)
    /// 
    /// # Returns
    /// Velocity vector tangent to the sphere surface
    pub fn get_velocity(position: Vec3) -> Vec3 {
        let speed = 7.0;
        
        // Calculate westward direction
        // West is the cross product of UP (Y-axis) with the radial position vector
        // This gives us the eastward direction, so we negate it for westward
        let up = Vec3::Y;
        let east = up.cross(position);
        let west = -east.normalize();
        
        west * speed
    }
}
