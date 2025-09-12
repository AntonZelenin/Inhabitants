use crate::core::state::GameState;
use crate::planet::resources::*;
use crate::planet::ui::systems::*;
use bevy::prelude::*;

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
                Update,
                (
                    handle_buttons,
                    detect_settings_changes,
                    update_settings_on_change,
                    update_main_area_content,
                    handle_arrow_toggle_change,
                    update_seed_display_on_change,
                )
                    .run_if(in_state(GameState::PlanetGeneration)),
            );
    }
}
