use bevy::prelude::*;

#[derive(Component)]
pub struct PlanetEntity;

#[derive(Component)]
pub struct ArrowEntity;

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