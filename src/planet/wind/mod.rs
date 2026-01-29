pub mod components;
pub mod systems;

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub struct ComputeWindPlugin;

impl Plugin for ComputeWindPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin)
            .add_systems(Update, (
                systems::cleanup_wind_texture_on_new_planet,
                systems::ensure_wind_texture,
                systems::handle_wind_tab_events,
            ).chain());
    }
}
