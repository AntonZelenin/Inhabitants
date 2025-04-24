use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::InGame)
                .load_collection::<ModelAssets>()
            ,
        );
    }
}

#[derive(AssetCollection, Resource)]
pub struct ModelAssets {
    #[asset(path = "models/medieval_village.glb#Scene0")]
    pub village: Handle<Scene>,
}
 