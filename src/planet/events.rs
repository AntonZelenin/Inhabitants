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

#[derive(Message)]
pub struct WindTabActiveEvent {
    pub active: bool,
}

#[derive(Message)]
pub struct TemperatureTabActiveEvent {
    pub active: bool,
}

#[derive(Message)]
pub struct PlanetSpawnedEvent;
