use bevy::prelude::*;

#[derive(Event)]
pub struct GeneratePlanetEvent;

#[derive(Event)]
pub struct GenerateNewSeedEvent;

#[derive(Event)]
pub struct ToggleArrowsEvent {
    pub show_arrows: bool,
}

#[derive(Event)]
pub struct SetCameraPositionEvent {
    pub position: Vec3,
}

#[derive(Event)]
pub struct SettingsChanged;
