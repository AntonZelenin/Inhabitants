use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

use crate::ui::bundles::*;
use crate::ui::components::*;

pub fn spawn_value_adjuster(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    initial_value: f32,
    min_value: f32,
    max_value: f32,
    step: f32,
    is_integer: bool,
) -> Entity {
    let container_node = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(5.0),
        width: Val::Percent(100.0),
        ..default()
    };

    parent
        .spawn((
            container_node,
            ValueAdjuster {
                current_value: initial_value,
                min_value,
                max_value,
                step,
                is_integer,
            },
        ))
        .with_children(|parent| {
            let adjuster_entity = parent.target_entity();

            // Label
            parent.spawn(
                LabelBundle::new(label, 16.0, Color::WHITE)
                    .with_margin(UiRect::bottom(Val::Px(5.0)))
            );

            // Value control row
            let control_row_node = Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            };

            parent
                .spawn(control_row_node)
                .with_children(|parent| {
                    let button_config = ButtonConfig {
                        normal_color: Color::srgb(0.5, 0.5, 0.5),
                        hover_color: Color::srgb(0.6, 0.6, 0.6),
                        pressed_color: Color::srgb(0.4, 0.4, 0.4),
                    };

                    // Decrement button
                    parent.spawn((
                        SmallButtonBundle::new(30.0, Color::srgb(0.5, 0.5, 0.5)),
                        UIButton,
                        DecrementButton,
                        AdjusterTarget(adjuster_entity),
                        button_config.clone(),
                    ))
                    .with_children(|parent| {
                        parent.spawn(LabelBundle::new("-", 18.0, Color::WHITE));
                    });

                    // Value display
                    let display_value = if is_integer {
                        format!("{}", initial_value as i32)
                    } else {
                        format!("{:.1}", initial_value)
                    };

                    let display_node = Node {
                        width: Val::Px(80.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    };

                    parent.spawn((
                        LabelBundle::new(&display_value, 16.0, Color::srgb(0.8, 0.8, 1.0))
                            .with_node(display_node),
                        ValueDisplay,
                        AdjusterTarget(adjuster_entity),
                    ));

                    // Increment button
                    parent.spawn((
                        SmallButtonBundle::new(30.0, Color::srgb(0.5, 0.5, 0.5)),
                        UIButton,
                        IncrementButton,
                        AdjusterTarget(adjuster_entity),
                        button_config,
                    ))
                    .with_children(|parent| {
                        parent.spawn(LabelBundle::new("+", 18.0, Color::WHITE));
                    });
                });
        })
        .id()
}

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