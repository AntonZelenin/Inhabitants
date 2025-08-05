pub mod components;

use bevy::prelude::*;
use components::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                handle_button_interactions,
                handle_toggle_interactions,
                handle_value_adjuster_interactions,
                update_toggle_text,
                update_value_displays,
            ),
        );
    }
}

fn handle_button_interactions(
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &ButtonConfig),
        (Changed<Interaction>, With<UIButton>),
    >,
) {
    for (interaction, mut bg_color, config) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(config.pressed_color);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(config.hover_color);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(config.normal_color);
            }
        }
    }
}

fn handle_toggle_interactions(
    mut toggle_query: Query<
        (&Interaction, &mut ToggleState, &mut BackgroundColor),
        (Changed<Interaction>, With<UIToggle>),
    >,
) {
    for (interaction, mut toggle_state, mut bg_color) in toggle_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            toggle_state.is_on = !toggle_state.is_on;
            *bg_color = if toggle_state.is_on {
                BackgroundColor(Color::srgb(0.3, 0.7, 0.3)) // Green when on
            } else {
                BackgroundColor(Color::srgb(0.6, 0.6, 0.6)) // Gray when off
            };
        }
    }
}

fn handle_value_adjuster_interactions(
    increment_query: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<IncrementButton>)>,
    decrement_query: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<DecrementButton>)>,
    mut adjuster_query: Query<&mut ValueAdjuster>,
) {
    // Handle increment buttons
    for (interaction, parent) in &increment_query {
        if *interaction == Interaction::Pressed {
            if let Ok(mut adjuster) = adjuster_query.get_mut(parent.get()) {
                adjuster.current_value =
                    (adjuster.current_value + adjuster.step).min(adjuster.max_value);
            }
        }
    }

    // Handle decrement buttons
    for (interaction, parent) in &decrement_query {
        if *interaction == Interaction::Pressed {
            if let Ok(mut adjuster) = adjuster_query.get_mut(parent.get()) {
                adjuster.current_value =
                    (adjuster.current_value - adjuster.step).max(adjuster.min_value);
            }
        }
    }
}

fn update_toggle_text(
    mut text_query: Query<&mut Text>,
    toggle_query: Query<(&ToggleState, &Children), (Changed<ToggleState>, With<UIToggle>)>,
) {
    for (toggle_state, children) in &toggle_query {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                text.0 = if toggle_state.is_on {
                    "ON".to_string()
                } else {
                    "OFF".to_string()
                };
            }
        }
    }
}

fn update_value_displays(
    mut text_query: Query<&mut Text, With<ValueDisplay>>,
    adjuster_query: Query<(&ValueAdjuster, &Children), Changed<ValueAdjuster>>,
) {
    for (adjuster, children) in &adjuster_query {
        for child in children.iter() {
            // Find the value display text in the children hierarchy
            if let Ok(mut text) = text_query.get_mut(child) {
                let display_value = if adjuster.is_integer {
                    format!("{}", adjuster.current_value as i32)
                } else {
                    format!("{:.1}", adjuster.current_value)
                };
                text.0 = display_value;
            }
        }
    }
}