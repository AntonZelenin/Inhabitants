use crate::core::state::GameState;
use crate::ui::components::*;
use crate::ui::widgets::*;
use bevy::prelude::*;

// Event for when any settings value changes
#[derive(Event)]
pub struct SettingsChanged;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetGenerationSettings>()
            .add_event::<SettingsChanged>()
            .add_systems(
                OnEnter(GameState::MainMenu),
                (setup_main_menu, add_markers).chain(),
            )
            .add_systems(OnExit(GameState::MainMenu), cleanup_menu)
            .add_systems(
                OnEnter(GameState::PlanetWithMenu),
                (setup_planet_with_menu, add_markers).chain(),
            )
            .add_systems(OnExit(GameState::PlanetWithMenu), cleanup_planet_menu)
            .add_systems(
                Update,
                (
                    handle_buttons,
                    detect_settings_changes,
                    update_settings_on_change,
                )
                    .run_if(in_state(GameState::MainMenu).or(in_state(GameState::PlanetWithMenu))),
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

#[derive(Component, Default)]
struct GeneratePlanetButton;

#[derive(Component, Default)]
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
                    // Planet Radius Slider (changed from value adjuster)
                    spawn_slider(
                        parent,
                        "Planet Radius",
                        settings.radius,
                        5.0,
                        50.0,
                        false,
                        350.0,
                    );

                    // Cells Per Unit Slider (changed from value adjuster)
                    spawn_slider(
                        parent,
                        "Cells Per Unit",
                        settings.cells_per_unit,
                        0.5,
                        5.0,
                        false,
                        350.0,
                    );

                    // Number of Plates Slider (changed from value adjuster)
                    spawn_slider(
                        parent,
                        "Number of Plates",
                        settings.num_plates as f32,
                        5.0,
                        30.0,
                        true,
                        350.0,
                    );

                    // Number of Micro Plates Slider (changed from value adjuster)
                    spawn_slider(
                        parent,
                        "Number of Micro Plates",
                        settings.num_micro_plates as f32,
                        0.0,
                        20.0,
                        true,
                        350.0,
                    );

                    // Show Arrows Toggle
                    spawn_toggle(parent, "Show Direction Arrows", settings.show_arrows);
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
                    spawn_button(
                        parent,
                        "Generate Planet",
                        Color::srgb(0.2, 0.7, 0.2),
                        Color::srgb(0.3, 0.8, 0.3),
                        Color::srgb(0.1, 0.6, 0.1),
                    );

                    // Quit button
                    spawn_button(
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
fn add_markers(
    mut commands: Commands,
    sliders: Query<(Entity, &Slider), (Without<RadiusSlider>, Without<PlanetRadiusSlider>)>,
    toggles: Query<(Entity, &ToggleState), (Without<ShowArrowsToggle>, Without<PlanetShowArrowsToggle>)>,
    buttons: Query<(Entity, &UIButton), (Without<GeneratePlanetButton>, Without<QuitButton>, Without<BackToMainMenuButton>)>,
    text_query: Query<&Text>,
    children_query: Query<&Children>,
) {
    // Add marker components to sliders based on their initial values
    for (entity, slider) in sliders.iter() {
        let value = slider.current_value;
        if (value - 20.0).abs() < 0.1 && !slider.is_integer {
            // Could be either main menu or planet menu radius slider
            commands.entity(entity).insert(RadiusSlider);
            commands.entity(entity).insert(PlanetRadiusSlider);
        } else if (value - 2.0).abs() < 0.1 && !slider.is_integer {
            commands.entity(entity).insert(CellsPerUnitSlider);
            commands.entity(entity).insert(PlanetCellsPerUnitSlider);
        } else if (value - 15.0).abs() < 0.1 && slider.is_integer {
            commands.entity(entity).insert(NumPlatesSlider);
            commands.entity(entity).insert(PlanetNumPlatesSlider);
        } else if (value - 5.0).abs() < 0.1 && slider.is_integer {
            commands.entity(entity).insert(NumMicroPlatesSlider);
            commands.entity(entity).insert(PlanetNumMicroPlatesSlider);
        }
    }

    // Add marker to toggle (there should be only one)
    for (entity, _) in toggles.iter() {
        commands.entity(entity).insert(ShowArrowsToggle);
        commands.entity(entity).insert(PlanetShowArrowsToggle);
    }

    // Add markers to buttons - we need to check their text content
    for (entity, _) in buttons.iter() {
        if let Ok(children) = children_query.get(entity) {
            for child in children.iter() {
                if let Ok(text) = text_query.get(child) {
                    if text.0.contains("Generate Planet") {
                        commands.entity(entity).insert(GeneratePlanetButton);
                        break;
                    } else if text.0.contains("Quit") {
                        commands.entity(entity).insert(QuitButton);
                        break;
                    } else if text.0.contains("Back to Main Menu") {
                        commands.entity(entity).insert(BackToMainMenuButton);
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

fn handle_buttons(
    generate_query: Query<&Interaction, (Changed<Interaction>, With<GeneratePlanetButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    back_query: Query<&Interaction, (Changed<Interaction>, With<BackToMainMenuButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    planet_entities: Query<Entity, With<crate::planet::PlanetEntity>>,
) {
    // Handle Generate Planet button
    for interaction in &generate_query {
        if *interaction == Interaction::Pressed {
            match current_state.get() {
                GameState::MainMenu => {
                    // From main menu: go to planet with menu
                    next_state.set(GameState::PlanetWithMenu);
                }
                GameState::PlanetWithMenu => {
                    // From planet menu: regenerate planet
                    // Despawn existing planet entities
                    for entity in planet_entities.iter() {
                        commands.entity(entity).despawn();
                    }
                    // Trigger planet regeneration by transitioning to InGame
                    next_state.set(GameState::InGame);
                }
                _ => {}
            }
        }
    }

    // Handle Quit button
    for interaction in &quit_query {
        if *interaction == Interaction::Pressed {
            app_exit_events.write(AppExit::Success);
        }
    }

    // Handle Back to Main Menu button
    for interaction in &back_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::MainMenu);
        }
    }
}

// System to detect settings changes and send event
fn detect_settings_changes(
    mut settings_changed_events: EventWriter<SettingsChanged>,
    radius_slider_query: Query<&Slider, (With<RadiusSlider>, Changed<Slider>)>,
    cells_slider_query: Query<&Slider, (With<CellsPerUnitSlider>, Changed<Slider>)>,
    plates_slider_query: Query<&Slider, (With<NumPlatesSlider>, Changed<Slider>)>,
    micro_plates_slider_query: Query<&Slider, (With<NumMicroPlatesSlider>, Changed<Slider>)>,
    toggle_query: Query<&ToggleState, (With<ShowArrowsToggle>, Changed<ToggleState>)>,
) {
    // Check if any slider or toggle has changed and send event
    let has_changes = !radius_slider_query.is_empty()
        || !cells_slider_query.is_empty()
        || !plates_slider_query.is_empty()
        || !micro_plates_slider_query.is_empty()
        || !toggle_query.is_empty();

    if has_changes {
        info!("HAS CHANGES!!!");
        settings_changed_events.write(SettingsChanged);
    }
}

fn update_settings_on_change(
    mut settings_changed_events: EventReader<SettingsChanged>,
    mut settings: ResMut<PlanetGenerationSettings>,
    radius_slider_query: Query<&Slider, With<RadiusSlider>>,
    cells_slider_query: Query<&Slider, With<CellsPerUnitSlider>>,
    plates_slider_query: Query<&Slider, With<NumPlatesSlider>>,
    micro_plates_slider_query: Query<&Slider, With<NumMicroPlatesSlider>>,
    toggle_query: Query<&ToggleState, With<ShowArrowsToggle>>,
) {
    // Only update settings if we received a change event
    for _ in settings_changed_events.read() {
        // Update settings from current slider and toggle values
        for slider in &radius_slider_query {
            settings.radius = slider.current_value;
        }
        for slider in &cells_slider_query {
            settings.cells_per_unit = slider.current_value;
        }
        for slider in &plates_slider_query {
            settings.num_plates = slider.current_value as usize;
        }
        for slider in &micro_plates_slider_query {
            settings.num_micro_plates = slider.current_value as usize;
        }
        for toggle_state in &toggle_query {
            settings.show_arrows = toggle_state.is_on;
        }
    }
}

#[derive(Component)]
struct PlanetMenuUI;

#[derive(Component)]
struct BackToMainMenuButton;

#[derive(Component)]
struct PlanetRadiusSlider;

#[derive(Component)]
struct PlanetCellsPerUnitSlider;

#[derive(Component)]
struct PlanetNumPlatesSlider;

#[derive(Component)]
struct PlanetNumMicroPlatesSlider;

#[derive(Component)]
struct PlanetShowArrowsToggle;

fn setup_planet_with_menu(mut commands: Commands, settings: Res<PlanetGenerationSettings>) {
    // Create a side panel layout instead of full screen
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            PlanetMenuUI,
        ))
        .with_children(|parent| {
            // Main 3D view area (left side)
            parent.spawn((
                Node {
                    width: Val::Percent(75.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE), // Transparent for 3D view
            ));

            // Settings panel (right side)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(25.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(20.0)),
                        row_gap: Val::Px(15.0),
                        justify_content: JustifyContent::FlexStart,
                        align_items: AlignItems::Stretch,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)), // Semi-transparent dark background
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((
                        Text::new("Planet Settings"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Node {
                            margin: UiRect::bottom(Val::Px(20.0)),
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                    ));

                    // Planet Radius Slider
                    spawn_slider(
                        parent,
                        "Planet Radius",
                        settings.radius,
                        5.0,
                        50.0,
                        false,
                        200.0,
                    );

                    // Cells Per Unit Slider
                    spawn_slider(
                        parent,
                        "Cells Per Unit",
                        settings.cells_per_unit,
                        0.5,
                        5.0,
                        false,
                        200.0,
                    );

                    // Number of Plates Slider
                    spawn_slider(
                        parent,
                        "Number of Plates",
                        settings.num_plates as f32,
                        5.0,
                        30.0,
                        true,
                        200.0,
                    );

                    // Number of Micro Plates Slider
                    spawn_slider(
                        parent,
                        "Number of Micro Plates",
                        settings.num_micro_plates as f32,
                        0.0,
                        20.0,
                        true,
                        200.0,
                    );

                    // Show Arrows Toggle
                    spawn_toggle(parent, "Show Direction Arrows", settings.show_arrows);

                    // Spacer
                    parent.spawn(Node {
                        height: Val::Px(20.0),
                        ..default()
                    });

                    // Generate Planet button (changed from Regenerate)
                    spawn_button(
                        parent,
                        "Generate Planet",
                        Color::srgb(0.2, 0.7, 0.2),
                        Color::srgb(0.3, 0.8, 0.3),
                        Color::srgb(0.1, 0.6, 0.1),
                    );

                    // Back to Main Menu button
                    spawn_button(
                        parent,
                        "Back to Main Menu",
                        Color::srgb(0.7, 0.2, 0.2),
                        Color::srgb(0.8, 0.3, 0.3),
                        Color::srgb(0.6, 0.1, 0.1),
                    );
                });
        });
}

fn cleanup_planet_menu(mut commands: Commands, query: Query<Entity, With<PlanetMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}