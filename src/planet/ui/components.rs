use bevy::prelude::*;

#[derive(Component)]
pub struct WorldGenerationMenu;

#[derive(Component, Default)]
pub struct GeneratePlanetButton;

#[derive(Component, Default)]
pub struct QuitButton;

#[derive(Component)]
pub struct RadiusSlider;

#[derive(Component)]
pub struct CellsPerUnitSlider;

#[derive(Component)]
pub struct NumPlatesSlider;

#[derive(Component)]
pub struct NumMicroPlatesSlider;

#[derive(Component)]
pub struct ShowArrowsToggle;

#[derive(Component)]
pub struct SeedDisplay;

#[derive(Component)]
pub struct RandomSeedButton;

#[derive(Component)]
pub struct MainArea;

#[derive(Component)]
pub struct PlaceholderText;
