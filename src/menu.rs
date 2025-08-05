use crate::core::state::GameState;
use crate::ui::components::*;
use bevy::prelude::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetGenerationSettings>()
            .add_systems(
                OnEnter(GameState::MainMenu),
                (setup_main_menu, add_menu_markers).chain(),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
            .add_systems(
                Update,
                (handle_menu_buttons, sync_settings_with_sliders)
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

fn setup_main_menu(mut commands: Commands, settings: Res<PlanetGenerationSettings>) {
    // Create all the UI elements using the existing functions
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
                    // Planet Radius Value Adjuster
                    create_value_adjuster(
                        parent,
                        "Planet Radius",
                        settings.radius,
                        5.0,
                        50.0,
                        1.0,
                        false,
                    );

                    // Cells Per Unit Value Adjuster
                    create_value_adjuster(
                        parent,
                        "Cells Per Unit",
                        settings.cells_per_unit,
                        0.5,
                        5.0,
                        0.1,
                        false,
                    );

                    // Number of Plates Value Adjuster
                    create_value_adjuster(
                        parent,
                        "Number of Plates",
                        settings.num_plates as f32,
                        5.0,
                        30.0,
                        1.0,
                        true,
                    );

                    // Number of Micro Plates Value Adjuster
                    create_value_adjuster(
                        parent,
                        "Number of Micro Plates",
                        settings.num_micro_plates as f32,
                        0.0,
                        20.0,
                        1.0,
                        true,
                    );

                    // Show Arrows Toggle
                    create_toggle(parent, "Show Direction Arrows", settings.show_arrows);
                });

            // Buttons container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    margin: UiRect::top(Val::Px(30.0)),
                    ..default()
                })
                .with_children(|parent| {
                    // Generate Planet button
                    create_button(
                        parent,
                        "Generate Planet",
                        Color::srgb(0.2, 0.7, 0.2),
                        Color::srgb(0.3, 0.8, 0.3),
                        Color::srgb(0.1, 0.6, 0.1),
                    );

                    // Quit button
                    create_button(
                        parent,
                        "Quit",
                        Color::srgb(0.7, 0.2, 0.2),
                        Color::srgb(0.8, 0.3, 0.3),
                        Color::srgb(0.6, 0.1, 0.1),
                    );
                });
        });
}

// System to add marker components after UI is created
fn add_menu_markers(
    mut commands: Commands,
    value_adjusters: Query<(Entity, &ValueAdjuster), Without<RadiusSlider>>,
    toggles: Query<(Entity, &ToggleState), Without<ShowArrowsToggle>>,
    buttons: Query<(Entity, &UIButton), (Without<GeneratePlanetButton>, Without<QuitButton>)>,
    text_query: Query<&Text>,
    children_query: Query<&Children>,
) {
    // Add marker components to value adjusters based on their initial values
    let mut adjusters: Vec<_> = value_adjusters.iter().collect();
    adjusters.sort_by(|a, b| a.1.current_value.partial_cmp(&b.1.current_value).unwrap());

    if adjusters.len() >= 4 {
        // Based on default values:
        // cells_per_unit: 2.0, num_micro_plates: 5.0, num_plates: 15.0, radius: 20.0

        // Find adjusters by their initial values
        for (entity, adjuster) in &adjusters {
            let value = adjuster.current_value;
            if (value - 2.0).abs() < 0.1 && !adjuster.is_integer {
                commands.entity(*entity).insert(CellsPerUnitSlider);
            } else if (value - 5.0).abs() < 0.1 && adjuster.is_integer {
                commands.entity(*entity).insert(NumMicroPlatesSlider);
            } else if (value - 15.0).abs() < 0.1 && adjuster.is_integer {
                commands.entity(*entity).insert(NumPlatesSlider);
            } else if (value - 20.0).abs() < 0.1 && !adjuster.is_integer {
                commands.entity(*entity).insert(RadiusSlider);
            }
        }
    }

    // Add marker to toggle (there should be only one)
    for (entity, _) in toggles.iter() {
        commands.entity(entity).insert(ShowArrowsToggle);
    }

    // Add markers to buttons - we need to check their text content
    for (entity, _) in buttons.iter() {
        // Check if this button has "Generate Planet" or "Quit" text in its children
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(text) = text_query.get(child) {
                    if text.0.contains("Generate Planet") {
                        commands.entity(entity).insert(GeneratePlanetButton);
                        break;
                    } else if text.0.contains("Quit") {
                        commands.entity(entity).insert(QuitButton);
                        break;
                    }
                }
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MainMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_menu_buttons(
    generate_query: Query<&Interaction, (Changed<Interaction>, With<GeneratePlanetButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    // Handle Generate Planet button
    for interaction in &generate_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::InGame);
        }
    }

    // Handle Quit button
    for interaction in &quit_query {
        if *interaction == Interaction::Pressed {
            app_exit_events.write(AppExit::Success);
        }
    }
}

fn sync_settings_with_sliders(
    radius_query: Query<&ValueAdjuster, (With<RadiusSlider>, Changed<ValueAdjuster>)>,
    cells_query: Query<&ValueAdjuster, (With<CellsPerUnitSlider>, Changed<ValueAdjuster>)>,
    plates_query: Query<&ValueAdjuster, (With<NumPlatesSlider>, Changed<ValueAdjuster>)>,
    micro_plates_query: Query<&ValueAdjuster, (With<NumMicroPlatesSlider>, Changed<ValueAdjuster>)>,
    toggle_query: Query<&ToggleState, (With<ShowArrowsToggle>, Changed<ToggleState>)>,
    mut settings: ResMut<PlanetGenerationSettings>,
) {
    // Update radius
    for adjuster in &radius_query {
        settings.radius = adjuster.current_value;
    }

    // Update cells per unit
    for adjuster in &cells_query {
        settings.cells_per_unit = adjuster.current_value;
    }

    // Update number of plates
    for adjuster in &plates_query {
        settings.num_plates = adjuster.current_value as usize;
    }

    // Update number of micro plates
    for adjuster in &micro_plates_query {
        settings.num_micro_plates = adjuster.current_value as usize;
    }

    // Update show arrows toggle
    for toggle_state in &toggle_query {
        settings.show_arrows = toggle_state.is_on;
    }
}

// Helper functions that return component bundles instead of creating entities
fn create_value_adjuster_components(
    label: &str,
    initial_value: f32,
    min_value: f32,
    max_value: f32,
    step: f32,
    is_integer: bool,
) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(5.0),
            width: Val::Percent(100.0),
            ..default()
        },
        ValueAdjuster {
            current_value: initial_value,
            min_value,
            max_value,
            step,
            is_integer,
        },
    )
}

fn create_toggle_components(label: &str, initial_state: bool) -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            width: Val::Percent(100.0),
            ..default()
        },
        ToggleState {
            is_on: initial_state,
        },
    )
}

fn create_button_components(
    text: &str,
    normal_color: Color,
    hover_color: Color,
    pressed_color: Color,
) -> impl Bundle {
    (
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
    )
}