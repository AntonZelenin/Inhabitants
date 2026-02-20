pub mod systems;

use bevy::prelude::*;

pub struct BiomePlugin;

impl Plugin for BiomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<systems::BiomeColorState>()
            .add_systems(Update, systems::update_continent_biome_colors);
    }
}
