use bevy::prelude::States;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub(crate) enum GameState {
    #[default]
    AssetLoading,
    Loading,
    LoadingSaveFile,
    // I'm creating this state because there's a bug in reading events from world
    // and if I use save event the system runs multiple times
    SaveGame,
    InGame,
    MainMenu,
}
