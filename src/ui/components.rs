use bevy::prelude::*;

#[derive(Component)]
pub struct UIButton;

#[derive(Component, Clone)]
pub struct ButtonConfig {
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
}

#[derive(Component)]
pub struct UIToggle;

#[derive(Component)]
pub struct ToggleState {
    pub is_on: bool,
}

#[derive(Component)]
pub struct ValueAdjuster {
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub step: f32,
    pub is_integer: bool,
}

#[derive(Component)]
pub struct DecrementButton;

#[derive(Component)]
pub struct IncrementButton;

#[derive(Component)]
pub struct ValueDisplay;

#[derive(Component)]
pub struct AdjusterTarget(pub Entity);

#[derive(Component)]
pub struct Slider {
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub is_integer: bool,
}

#[derive(Component)]
pub struct SliderTrack;

#[derive(Component)]
pub struct SliderHandle;

#[derive(Component)]
pub struct SliderTarget(pub Entity);

#[derive(Component)]
pub struct SliderValueDisplay;
