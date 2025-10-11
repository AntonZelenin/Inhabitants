use bevy::prelude::*;

#[derive(Message)]
pub struct GeneratePlanetEvent;

#[derive(Message)]
pub struct GenerateNewSeedEvent;

#[derive(Message)]
pub struct ToggleArrowsEvent {
    pub show_arrows: bool,
}

#[derive(Message)]
pub struct SetCameraPositionEvent {
    pub position: Vec3,
}

#[derive(Message)]
pub struct SettingsChanged;
