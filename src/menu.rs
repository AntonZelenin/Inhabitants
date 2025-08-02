use crate::core::state::GameState;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetGenerationSettings>()
            .add_systems(OnEnter(GameState::MainMenu), setup_main_menu)
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
            .add_systems(
                Update,
                (
                    main_menu_system,
                    planet_settings_system,
                    update_slider_values,
                )
                    .run_if(in_state(GameState::MainMenu)),
            );
    }
}

#[derive(Resource, Clone)]
pub struct PlanetGenerationSettings {
    pub radius: f32,
    pub cells_per_unit: f32,
    pub num_plates: usize,
    pub num_micro_plates: usize,
    pub show_arrows: bool,
}

impl Default for PlanetGenerationSettings {
    fn default() -> Self {
        Self {
            radius: 20.0,
            cells_per_unit: 2.0,
            num_plates: 15,
            num_micro_plates: 5,
            show_arrows: true,
        }
    }
}

#[derive(Component)]
struct MainMenuUI;

#[derive(Component)]
struct GeneratePlanetButton;

#[derive(Component)]
struct QuitButton;

#[derive(Component)]
struct RadiusSlider;

#[derive(Component)]
struct CellsPerUnitSlider;

#[derive(Component)]
struct NumPlatesSlider;

#[derive(Component)]
struct NumMicroPlatesSlider;

#[derive(Component)]
struct ShowArrowsToggle;

#[derive(Component)]
struct ToggleBackground;

#[derive(Component)]
struct SettingValue;

fn setup_main_menu(mut commands: Commands) {
    // Main menu container
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            MainMenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Planet Generation Settings"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Settings panel
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(15.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                    BorderRadius::all(Val::Px(10.0)),
                ))
                .with_children(|parent| {
                    // Planet Radius
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Label and value row
                            parent
                                .spawn((Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Planet Radius"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));

                                    parent.spawn((
                                        Text::new("20.0"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                        SettingValue,
                                    ));
                                });

                            // Slider
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                                BorderRadius::all(Val::Px(10.0)),
                                RadiusSlider,
                            ));
                        });

                    // Cells Per Unit
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Label and value row
                            parent
                                .spawn((Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Cells Per Unit"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));

                                    parent.spawn((
                                        Text::new("2.0"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                        SettingValue,
                                    ));
                                });

                            // Slider
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                                BorderRadius::all(Val::Px(10.0)),
                                CellsPerUnitSlider,
                            ));
                        });

                    // Number of Plates
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Label and value row
                            parent
                                .spawn((Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Number of Plates"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));

                                    parent.spawn((
                                        Text::new("15"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                        SettingValue,
                                    ));
                                });

                            // Slider
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                                BorderRadius::all(Val::Px(10.0)),
                                NumPlatesSlider,
                            ));
                        });

                    // Number of Micro Plates
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
                            width: Val::Percent(100.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            // Label and value row
                            parent
                                .spawn((Node {
                                    flex_direction: FlexDirection::Row,
                                    justify_content: JustifyContent::SpaceBetween,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Number of Micro Plates"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));

                                    parent.spawn((
                                        Text::new("5"),
                                        TextFont {
                                            font_size: 16.0,
                                            ..default()
                                        },
                                        TextColor(Color::srgb(0.8, 0.8, 1.0)),
                                        SettingValue,
                                    ));
                                });

                            // Slider
                            parent.spawn((
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(20.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                                BorderRadius::all(Val::Px(10.0)),
                                NumMicroPlatesSlider,
                            ));
                        });

                    // Show Arrows Toggle
                    parent
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::SpaceBetween,
                            width: Val::Percent(100.0),
                            ..default()
                        },))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Show Direction Arrows"),
                                TextFont {
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));

                            parent
                                .spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(60.0),
                                        height: Val::Px(30.0),
                                        align_items: AlignItems::Center,
                                        justify_content: JustifyContent::Center,
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
                                    BorderRadius::all(Val::Px(5.0)),
                                    ShowArrowsToggle,
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("ON"),
                                        TextFont {
                                            font_size: 14.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });

            // Buttons
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                },))
                .with_children(|parent| {
                    // Generate Planet Button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(150.0),
                                height: Val::Px(50.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.7, 0.2)),
                            BorderRadius::all(Val::Px(8.0)),
                            GeneratePlanetButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Generate Planet"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                    // Quit Button
                    parent
                        .spawn((
                            Button,
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(50.0),
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.7, 0.2, 0.2)),
                            BorderRadius::all(Val::Px(8.0)),
                            QuitButton,
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Quit"),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });
        });
}

fn main_menu_system(
    mut button_queries: ParamSet<(
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<GeneratePlanetButton>)>,
        Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<QuitButton>)>,
        Query<&mut BackgroundColor, With<ShowArrowsToggle>>,
    )>,
    toggle_query: Query<&Interaction, (With<ShowArrowsToggle>, Changed<Interaction>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut settings: ResMut<PlanetGenerationSettings>,
    mut app_exit_events: EventWriter<AppExit>,
    mut toggle_text_query: Query<&mut Text>,
) {
    // Handle Generate Planet button interactions
    for (interaction, mut color) in button_queries.p0().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::InGame);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.3, 0.8, 0.3)); // Lighter green
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.2, 0.7, 0.2)); // Original green
            }
        }
    }

    // Handle Quit button interactions
    for (interaction, mut color) in button_queries.p1().iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                app_exit_events.write(AppExit::Success);
            }
            Interaction::Hovered => {
                *color = BackgroundColor(Color::srgb(0.8, 0.3, 0.3)); // Lighter red
            }
            Interaction::None => {
                *color = BackgroundColor(Color::srgb(0.7, 0.2, 0.2)); // Original red
            }
        }
    }

    // Show Arrows Toggle
    for interaction in &toggle_query {
        if *interaction == Interaction::Pressed {
            settings.show_arrows = !settings.show_arrows;

            // Update toggle appearance using the ParamSet
            if let Ok(mut bg_color) = button_queries.p2().single_mut() {
                *bg_color = if settings.show_arrows {
                    BackgroundColor(Color::srgb(0.3, 0.7, 0.3))
                } else {
                    BackgroundColor(Color::srgb(0.7, 0.3, 0.3))
                };
            }

            // Update toggle text
            for mut text in &mut toggle_text_query {
                if text.0 == "ON" || text.0 == "OFF" {
                    text.0 = if settings.show_arrows {
                        "ON".to_string()
                    } else {
                        "OFF".to_string()
                    };
                }
            }
        }
    }
}

fn planet_settings_system(// This is a placeholder for more complex slider interactions
    // For now, we'll use simple button-based value changes
) {
    // TODO: Implement slider drag interactions for fine-tuning values
}

fn update_slider_values(// This system will update the displayed values based on slider positions
    // For now, this is a placeholder for the slider value update logic
) {
    // TODO: Implement slider value updates
}

fn cleanup_menu(mut commands: Commands, menu_query: Query<Entity, With<MainMenuUI>>) {
    for entity in &menu_query {
        commands.entity(entity).despawn();
    }
}