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

#[derive(Message, Clone, Copy, Debug, PartialEq)]
pub enum ViewTabType {
    Continent,
    Tectonic,
    Wind,
    Temperature,
    Precipitations,
    Biomes,
}

#[derive(Message)]
pub struct TabSwitchEvent {
    pub tab: ViewTabType,
}

#[derive(Message)]
pub struct WindTabActiveEvent {
    pub active: bool,
}

#[derive(Message)]
pub struct TectonicTabActiveEvent {
    pub active: bool,
}

#[derive(Message)]
pub struct TemperatureTabActiveEvent {
    pub active: bool,
}

#[derive(Message)]
pub struct PrecipitationTabActiveEvent {
    pub active: bool,
}

#[derive(Message)]
pub struct PlanetSpawnedEvent;

#[derive(Message)]
pub struct ResetCameraEvent;
