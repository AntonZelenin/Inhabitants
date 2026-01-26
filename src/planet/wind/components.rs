use bevy::prelude::*;

/// Wind particle component - stores particle state for wind visualization
#[derive(Component)]
pub struct WindParticle {
    /// Current position on sphere (normalized direction vector)
    pub position: Vec3,
    /// Velocity vector (tangent to sphere)
    pub velocity: Vec3,
    /// Age of particle (for fading/recycling)
    pub age: f32,
    /// Maximum lifetime before respawn
    pub lifetime: f32,
    /// Unique particle ID for stable respawning
    pub particle_id: u32,
    /// Number of times this particle has respawned (for randomization)
    pub respawn_count: u32,
    /// Trail history - stores previous positions for curved trail rendering
    pub trail_positions: std::collections::VecDeque<Vec3>,
}

/// Marker for wind visualization entities
#[derive(Component)]
pub struct WindView;
