use bevy::prelude::*;

#[derive(Component)]
pub struct PlanetEntity;

#[derive(Component)]
pub struct OceanEntity;

#[derive(Component)]
pub struct ContinentViewMesh;

#[derive(Component)]
pub struct PlateViewMesh;

#[derive(Component)]
pub struct ArrowEntity;

/// Marker component for entities that should only be visible in continent view mode
#[derive(Component)]
pub struct ContinentView;

/// Marker component for entities that should only be visible in tectonic plate view mode
#[derive(Component)]
pub struct TectonicPlateView;

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
}

/// Marker for wind visualization entities
#[derive(Component)]
pub struct WindView;

#[derive(Component)]
pub struct PlanetControls {
    pub rotation: Quat,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

#[derive(Component)]
pub struct CameraLerp {
    pub target_position: Vec3,
    pub target_look_at: Vec3,
    pub current_look_at: Vec3,
    pub pivot: Vec3,
    pub dir: Vec3,
    pub lerp_speed: f32,
    pub is_lerping: bool,
}