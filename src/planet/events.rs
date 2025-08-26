use bevy::prelude::*;

#[derive(Event)]
pub struct GeneratePlanetEvent;

#[derive(Event)]
pub struct ToggleArrowsEvent {
    pub show_arrows: bool,
}
