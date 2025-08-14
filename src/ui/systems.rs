use crate::ui::components::*;
use bevy::color::Color;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;

pub fn handle_button_interactions(
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

pub fn handle_toggle_interactions(
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

pub fn handle_value_adjuster_interactions(
    increment_query: Query<
        (&Interaction, &AdjusterTarget),
        (Changed<Interaction>, With<IncrementButton>),
    >,
    decrement_query: Query<
        (&Interaction, &AdjusterTarget),
        (Changed<Interaction>, With<DecrementButton>),
    >,
    mut adjuster_query: Query<&mut ValueAdjuster>,
) {
    // Handle increment buttons
    for (interaction, target) in &increment_query {
        if *interaction == Interaction::Pressed {
            if let Ok(mut adjuster) = adjuster_query.get_mut(target.0) {
                adjuster.current_value =
                    (adjuster.current_value + adjuster.step).min(adjuster.max_value);
            }
        }
    }

    // Handle decrement buttons
    for (interaction, target) in &decrement_query {
        if *interaction == Interaction::Pressed {
            if let Ok(mut adjuster) = adjuster_query.get_mut(target.0) {
                adjuster.current_value =
                    (adjuster.current_value - adjuster.step).max(adjuster.min_value);
            }
        }
    }
}

pub fn update_toggle_text(
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

pub fn update_value_displays(
    mut text_query: Query<&mut Text, With<ValueDisplay>>,
    display_query: Query<(Entity, &AdjusterTarget), With<ValueDisplay>>,
    adjuster_query: Query<(Entity, &ValueAdjuster), Changed<ValueAdjuster>>,
) {
    // For each changed adjuster, find its displays and update them
    for (adjuster_entity, adjuster) in &adjuster_query {
        for (display_entity, target) in &display_query {
            if target.0 == adjuster_entity {
                if let Ok(mut text) = text_query.get_mut(display_entity) {
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
}

pub fn handle_slider_interactions(
    mut slider_handle_query: Query<
        (&Interaction, &SliderTarget),
        (Changed<Interaction>, With<SliderHandle>),
    >,
    mut slider_query: Query<&mut Slider>,
    track_query: Query<(&Node, &SliderTarget), (With<SliderTrack>, Without<SliderHandle>)>,
    windows: Query<&Window>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut drag_state: Local<Option<Entity>>, // Just store which slider is being dragged
) {
    let window = windows.single().expect("Expected a single window");

    // Stop dragging immediately on mouse release
    if !mouse_input.pressed(MouseButton::Left) {
        *drag_state = None;
        // drain events
        for _ in mouse_motion.read() {}
        return;
    }

    // Start/stop drag based on handle interaction
    for (interaction, target) in slider_handle_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                *drag_state = Some(target.0);
                // drain stale events
                for _ in mouse_motion.read() {}
            }
            Interaction::None => {
                if matches!(*drag_state, Some(id) if id == target.0) {
                    *drag_state = None;
                }
            }
            _ => {}
        }
    }

    // While dragging, update using mouse delta for movement
    if let Some(slider_entity) = *drag_state {
        let mut dx = 0.0;
        for ev in mouse_motion.read() {
            dx += ev.delta.x;
        }

        // Update slider even if dx is 0 (for initial click without movement)
        if let Ok(mut slider) = slider_query.get_mut(slider_entity) {
            if let Some((track_node, _)) = track_query.iter().find(|(_, t)| t.0 == slider_entity) {
                if let Val::Px(track_width) = track_node.width {
                    let handle_size = 18.0;
                    let usable = (track_width - handle_size).max(0.0);

                    // Convert current value to position
                    let current_ratio = (slider.current_value - slider.min_value)
                        / (slider.max_value - slider.min_value);
                    let current_left = current_ratio * usable;

                    // Apply mouse delta (will be 0 on initial click, but that's fine)
                    let new_left = (current_left + dx).clamp(0.0, usable);

                    // Convert back to value
                    let new_ratio = if usable > 0.0 { new_left / usable } else { 0.0 };
                    let range = slider.max_value - slider.min_value;
                    let new_value = slider.min_value + new_ratio * range;

                    slider.current_value = if slider.is_integer {
                        new_value.round().clamp(slider.min_value, slider.max_value)
                    } else {
                        new_value.clamp(slider.min_value, slider.max_value)
                    };
                }
            }
        }
    }
}

pub fn update_slider_handles(
    slider_query: Query<(Entity, &Slider), Changed<Slider>>,
    mut handle_query: Query<(&SliderTarget, &mut Node), With<SliderHandle>>,
    track_query: Query<&Node, (With<SliderTrack>, Without<SliderHandle>)>,
) {
    for (slider_entity, slider) in slider_query.iter() {
        // Find the track width for this slider
        let track_width = track_query
            .iter()
            .find_map(|track_node| {
                if let Val::Px(width) = track_node.width {
                    Some(width)
                } else {
                    None
                }
            })
            .unwrap_or(200.0);

        let handle_size = 18.0; // Fixed: match the actual handle size from widget
        let value_ratio =
            (slider.current_value - slider.min_value) / (slider.max_value - slider.min_value);
        let handle_position = value_ratio * (track_width - handle_size);

        // Update handle position
        for (target, mut handle_node) in handle_query.iter_mut() {
            if target.0 == slider_entity {
                handle_node.left = Val::Px(handle_position);
            }
        }
    }
}

pub fn update_slider_value_displays(
    mut text_query: Query<&mut Text, With<SliderValueDisplay>>,
    display_query: Query<(Entity, &SliderTarget), With<SliderValueDisplay>>,
    slider_query: Query<(Entity, &Slider), Changed<Slider>>,
) {
    // For each changed slider, find its displays and update them
    for (slider_entity, slider) in slider_query.iter() {
        for (display_entity, target) in display_query.iter() {
            if target.0 == slider_entity {
                if let Ok(mut text) = text_query.get_mut(display_entity) {
                    let display_value = if slider.is_integer {
                        format!("{}", slider.current_value as i32)
                    } else {
                        format!("{:.1}", slider.current_value)
                    };
                    text.0 = display_value;
                }
            }
        }
    }
}