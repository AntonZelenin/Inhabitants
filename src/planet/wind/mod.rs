pub mod components;
pub mod systems;

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        // Wind systems will be registered from parent module
    }
}
