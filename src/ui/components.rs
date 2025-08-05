use bevy::ecs::relationship::RelatedSpawnerCommands;
use bevy::prelude::*;

// Button Components
#[derive(Component)]
pub struct UIButton;

#[derive(Component)]
pub struct ButtonConfig {
    pub normal_color: Color,
    pub hover_color: Color,
    pub pressed_color: Color,
}

// Toggle Components
#[derive(Component)]
pub struct UIToggle;

#[derive(Component)]
pub struct ToggleState {
    pub is_on: bool,
}

// Value adjuster components
#[derive(Component)]
pub struct ValueAdjuster {
    pub current_value: f32,
    pub min_value: f32,
    pub max_value: f32,
    pub step: f32,
    pub is_integer: bool,
}

#[derive(Component)]
pub struct DecrementButton;

#[derive(Component)]
pub struct IncrementButton;

#[derive(Component)]
pub struct ValueDisplay;

// Builder functions for creating UI components
pub fn create_value_adjuster(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    initial_value: f32,
    min_value: f32,
    max_value: f32,
    step: f32,
    is_integer: bool,
) -> Entity {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            width: Val::Percent(100.0),
            ..default()
        },))
        .with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(5.0)),
                    ..default()
                },
            ));

            // Value control row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|parent| {
                    // Decrement button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                            BorderRadius::all(Val::Px(5.0)),
                            Interaction::None,
                            UIButton,
                            DecrementButton,
                            ButtonConfig {
                                normal_color: Color::srgb(0.5, 0.5, 0.5),
                                hover_color: Color::srgb(0.6, 0.6, 0.6),
                                pressed_color: Color::srgb(0.4, 0.4, 0.4),
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("-"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Value display
                    let display_value = if is_integer {
                        format!("{}", initial_value as i32)
                    } else {
                        format!("{:.1}", initial_value)
                    };
                    parent.spawn((
                        Text::new(display_value),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                        Node {
                            width: Val::Px(80.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ValueDisplay,
                    ));

                    // Increment button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(30.0),
                                height: Val::Px(30.0),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.5, 0.5, 0.5)),
                            BorderRadius::all(Val::Px(5.0)),
                            Interaction::None,
                            UIButton,
                            IncrementButton,
                            ButtonConfig {
                                normal_color: Color::srgb(0.5, 0.5, 0.5),
                                hover_color: Color::srgb(0.6, 0.6, 0.6),
                                pressed_color: Color::srgb(0.4, 0.4, 0.4),
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("+"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        })
        .insert(ValueAdjuster {
            current_value: initial_value,
            min_value,
            max_value,
            step,
            is_integer,
        })
        .id()
}

pub fn create_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    text: &str,
    normal_color: Color,
    hover_color: Color,
    pressed_color: Color,
) -> Entity {
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(normal_color),
            BorderRadius::all(Val::Px(8.0)),
            Interaction::None,
            UIButton,
            ButtonConfig {
                normal_color,
                hover_color,
                pressed_color,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(text),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        })
        .id()
}

pub fn create_toggle(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    label: &str,
    initial_state: bool,
) -> Entity {
    parent
        .spawn((Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            width: Val::Percent(100.0),
            ..default()
        },))
        .with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Toggle button
            parent
                .spawn((
                    Button,
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(30.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(if initial_state {
                        Color::srgb(0.3, 0.7, 0.3)
                    } else {
                        Color::srgb(0.6, 0.6, 0.6)
                    }),
                    BorderRadius::all(Val::Px(15.0)),
                    Interaction::None,
                    UIToggle,
                    ToggleState {
                        is_on: initial_state,
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(if initial_state { "ON" } else { "OFF" }),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        })
        .id()
}