use crate::core::state::GameState;
use crate::planet::resources::*;
use crate::planet::ui::systems::*;
use bevy::prelude::*;

#[derive(Event)]
pub struct SettingsChanged;

#[derive(Event)]
pub struct ReceivedCharacter {
    pub char: char,
}

pub struct PlanetGenMenuPlugin;

impl Plugin for PlanetGenMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetGenerationSettings>()
            .add_event::<SettingsChanged>()
            .add_event::<ReceivedCharacter>()
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
                    generate_char_events,
                    handle_buttons,
                    detect_settings_changes,
                    update_settings_on_change,
                    update_main_area_content,
                    handle_arrow_toggle_change,
                    handle_seed_input,
                )
                    .run_if(in_state(GameState::PlanetGeneration)),
            );
    }
}
