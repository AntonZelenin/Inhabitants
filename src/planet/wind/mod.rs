pub mod components;

use bevy::prelude::*;

pub struct WindPlugin;

impl Plugin for WindPlugin {
    fn build(&self, app: &mut App) {
        // Wind systems will be registered from parent module
    }
}
