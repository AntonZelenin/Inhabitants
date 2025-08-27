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