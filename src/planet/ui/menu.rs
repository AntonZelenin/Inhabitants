use crate::core::state::GameState;
use crate::planet::resources::*;
use crate::planet::ui::systems::*;
use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

pub struct PlanetGenMenuPlugin;

impl Plugin for PlanetGenMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetGenerationSettings>()
            .add_systems(
                OnEnter(GameState::PlanetGeneration),
                setup_world_generation_menu,
            )
            .add_systems(
                OnExit(GameState::PlanetGeneration),
                cleanup_world_generation_menu,
            )
            .add_systems(
                EguiPrimaryContextPass,
                render_planet_generation_ui
                    .run_if(in_state(GameState::PlanetGeneration)),
            );
    }
}
