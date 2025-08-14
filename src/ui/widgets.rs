use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use crate::ui::bundles::*;
use crate::ui::components::*;

pub fn spawn_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: &str,
    normal_color: Color,
    hover_color: Color,
    pressed_color: Color,
) -> Entity {
    let button_node = Node {
        width: Val::Px(200.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button_config = ButtonConfig {
        normal_color,
        hover_color,
        pressed_color,
    };

    parent
        .spawn((
            Button,
            button_node,
            BackgroundColor(normal_color),
            BorderRadius::all(Val::Px(8.0)),
            Interaction::None,
            UIButton,
            button_config,
        ))
        .with_children(|parent| {
            parent.spawn(LabelBundle::new(text, 18.0, Color::WHITE));
        })
        .id()
}

pub fn spawn_toggle(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    initial_state: bool,
) -> Entity {
    let container_node = Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(10.0),
        width: Val::Percent(100.0),
        ..default()
    };

    parent
        .spawn(container_node)
        .with_children(|parent| {
            // Label
            parent.spawn(LabelBundle::new(label, 16.0, Color::WHITE));

            // Toggle button
            let toggle_button_node = Node {
                width: Val::Px(60.0),
                height: Val::Px(30.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            };

            let background_color = if initial_state {
                Color::srgb(0.3, 0.7, 0.3)
            } else {
                Color::srgb(0.6, 0.6, 0.6)
            };

            parent
                .spawn((
                    Button,
                    toggle_button_node,
                    BackgroundColor(background_color),
                    BorderRadius::all(Val::Px(15.0)),
                    Interaction::None,
                    UIToggle,
                    ToggleState {
                        is_on: initial_state,
                    },
                ))
                .with_children(|parent| {
                    let toggle_text = if initial_state { "ON" } else { "OFF" };

                    parent.spawn(LabelBundle::new(toggle_text, 14.0, Color::WHITE));
                });
        })
        .id()
}

pub fn spawn_slider(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    initial_value: f32,
    min_value: f32,
    max_value: f32,
    is_integer: bool,
    width: f32,
) -> Entity {
    let slider = Slider {
        current_value: initial_value,
        min_value,
        max_value,
        is_integer,
    };

    parent
        .spawn(SliderBundle::new(width, 80.0, slider))
        .with_children(|parent| {
            let slider_entity = parent.target_entity();

            // First row: Title on left + Value on right
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Percent(100.0),
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // Title on the left
                    parent.spawn(LabelBundle::new(label, 16.0, Color::WHITE));

                    // Current value on the right
                    let display_value = if is_integer {
                        format!("{}", initial_value as i32)
                    } else {
                        format!("{:.1}", initial_value)
                    };

                    parent.spawn((
                        LabelBundle::new(&display_value, 14.0, Color::srgb(0.8, 0.8, 1.0)),
                        SliderValueDisplay,
                        SliderTarget(slider_entity),
                    ));
                });

            // Second row: The slider track and controls
            let track_width = width - 40.0; // Margin for the track
            let track_height = 8.0;

            parent
                .spawn(Node {
                    width: Val::Px(width),
                    height: Val::Px(24.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // Left boundary marker
                    parent.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));

                    // Main track with visual improvements
                    parent
                        .spawn((
                            SliderTrackBundle::new(
                                track_width,
                                track_height,
                                Color::srgb(0.2, 0.2, 0.2),
                            ),
                            SliderTrack,
                            SliderTarget(slider_entity),
                        ))
                        .with_children(|parent| {
                            // Track center line
                            parent.spawn((
                                Node {
                                    width: Val::Px(track_width),
                                    height: Val::Px(1.0),
                                    position_type: PositionType::Absolute,
                                    top: Val::Px(track_height * 0.5 - 0.5),
                                    left: Val::Px(0.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                            ));

                            // Calculate handle position based on initial value
                            let handle_size = 18.0;
                            let value_ratio = (initial_value - min_value) / (max_value - min_value);
                            let handle_position = value_ratio * (track_width - handle_size);

                            parent.spawn((
                                SliderHandleBundle::new(handle_size, Color::srgb(0.8, 0.8, 1.0))
                                    .with_position(handle_position),
                                SliderHandle,
                                SliderTarget(slider_entity),
                            ));
                        });

                    // Right boundary marker
                    parent.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Px(16.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                });

            // Min/Max value labels (smaller, at the bottom)
            parent
                .spawn(Node {
                    width: Val::Px(width),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|parent| {
                    // Min value label
                    let min_text = if is_integer {
                        format!("{}", min_value as i32)
                    } else {
                        format!("{:.1}", min_value)
                    };
                    parent.spawn(LabelBundle::new(
                        &min_text,
                        12.0,
                        Color::srgb(0.6, 0.6, 0.6),
                    ));

                    // Max value label
                    let max_text = if is_integer {
                        format!("{}", max_value as i32)
                    } else {
                        format!("{:.1}", max_value)
                    };
                    parent.spawn(LabelBundle::new(
                        &max_text,
                        12.0,
                        Color::srgb(0.6, 0.6, 0.6),
                    ));
                });
        })
        .id()
}