use crate::core::state::GameState;
use crate::ui::components::*;
use crate::ui::widgets::*;
use bevy::prelude::*;
use crate::planet::components::PlanetEntity;

#[derive(Event)]
pub struct SettingsChanged;

pub struct PlanetGenMenuPlugin;

impl Plugin for PlanetGenMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlanetGenerationSettings>()
            .add_event::<SettingsChanged>()
            .add_systems(OnEnter(GameState::WorldGeneration), setup_world_generation_menu)
            .add_systems(OnExit(GameState::WorldGeneration), cleanup_world_generation_menu)
            .add_systems(
                Update,
                (
                    handle_buttons,
                    detect_settings_changes,
                    update_settings_on_change,
                    update_main_area_content,
                )
                    .run_if(in_state(GameState::WorldGeneration)),
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
struct WorldGenerationMenu;

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

#[derive(Component)]
struct MainArea;

#[derive(Component)]
struct PlaceholderText;

fn setup_world_generation_menu(mut commands: Commands, settings: Res<PlanetGenerationSettings>) {
    // Create a side panel layout instead of full screen
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            WorldGenerationMenu,
        ))
        .with_children(|parent| {
            // Main 3D view area (left side)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(75.0),
                        height: Val::Percent(100.0),
                        display: Display::Flex,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.1)), // Dark background
                    MainArea,
                ))
                .with_children(|parent| {
                    // Placeholder text shown when no planet exists
                    parent.spawn((
                        Text::new("Press Generate to see the preview"),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6)),
                        PlaceholderText,
                    ));
                });

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
                    spawn_slider_with_marker(
                        parent,
                        "Planet Radius",
                        settings.radius,
                        5.0,
                        50.0,
                        false,
                        200.0,
                        RadiusSlider,
                    );

                    // Cells Per Unit Slider
                    spawn_slider_with_marker(
                        parent,
                        "Cells Per Unit",
                        settings.cells_per_unit,
                        0.5,
                        5.0,
                        false,
                        200.0,
                        CellsPerUnitSlider,
                    );

                    // Number of Plates Slider
                    spawn_slider_with_marker(
                        parent,
                        "Number of Plates",
                        settings.num_plates as f32,
                        5.0,
                        30.0,
                        true,
                        200.0,
                        NumPlatesSlider,
                    );

                    // Number of Micro Plates Slider
                    spawn_slider_with_marker(
                        parent,
                        "Number of Micro Plates",
                        settings.num_micro_plates as f32,
                        0.0,
                        20.0,
                        true,
                        200.0,
                        NumMicroPlatesSlider,
                    );

                    // Show Arrows Toggle
                    spawn_toggle_with_marker(
                        parent,
                        "Show Direction Arrows",
                        settings.show_arrows,
                        ShowArrowsToggle,
                    );

                    // Spacer
                    parent.spawn(Node {
                        height: Val::Px(20.0),
                        ..default()
                    });

                    // Generate Planet button
                    spawn_button_with_marker(
                        parent,
                        "Generate Planet",
                        Color::srgb(0.2, 0.7, 0.2),
                        Color::srgb(0.3, 0.8, 0.3),
                        Color::srgb(0.1, 0.6, 0.1),
                        GeneratePlanetButton,
                    );

                    // Quit button
                    spawn_button_with_marker(
                        parent,
                        "Quit",
                        Color::srgb(0.7, 0.2, 0.2),
                        Color::srgb(0.8, 0.3, 0.3),
                        Color::srgb(0.6, 0.1, 0.1),
                        QuitButton,
                    );
                });
        });
}

fn cleanup_world_generation_menu(mut commands: Commands, query: Query<Entity, With<WorldGenerationMenu>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_buttons(
    generate_query: Query<&Interaction, (Changed<Interaction>, With<GeneratePlanetButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: EventWriter<AppExit>,
    current_state: Res<State<GameState>>,
    mut commands: Commands,
    planet_entities: Query<Entity, With<PlanetEntity>>,
) {
    // Handle Generate Planet button
    for interaction in &generate_query {
        if *interaction == Interaction::Pressed {
            match current_state.get() {
                GameState::WorldGeneration => {
                    // Despawn existing planet entities before generating new ones
                    for entity in planet_entities.iter() {
                        commands.entity(entity).despawn();
                    }
                    // Trigger planet generation by transitioning to InGame
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

fn update_main_area_content(
    planet_entities: Query<Entity, With<PlanetEntity>>,
    mut placeholder_query: Query<&mut Node, With<PlaceholderText>>,
    mut main_area_query: Query<&mut BackgroundColor, With<MainArea>>,
) {
    let has_planet = !planet_entities.is_empty();

    // Show/hide placeholder text based on planet existence
    for mut placeholder_node in placeholder_query.iter_mut() {
        placeholder_node.display = if has_planet {
            Display::None
        } else {
            Display::Flex
        };
    }

    // Update main area background - transparent when planet exists, dark when not
    for mut bg_color in main_area_query.iter_mut() {
        *bg_color = if has_planet {
            BackgroundColor(Color::NONE) // Transparent for 3D view
        } else {
            BackgroundColor(Color::srgb(0.05, 0.05, 0.1)) // Dark background
        };
    }
}