mod bundles;
pub mod components;
mod systems;
pub mod widgets;

use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_button_interactions,
                systems::handle_slider_interactions,
                systems::handle_toggle_interactions,
                systems::handle_value_adjuster_interactions,
                systems::update_toggle_text,
                systems::update_value_displays,
                systems::update_slider_handles,
                systems::update_slider_value_displays,
            ),
        );
    }
}
