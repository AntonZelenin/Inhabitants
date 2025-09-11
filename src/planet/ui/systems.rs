use crate::planet::components::PlanetEntity;
use crate::planet::events::{GeneratePlanetEvent, ToggleArrowsEvent};
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::ui::components::*;
use crate::planet::ui::menu::{ReceivedCharacter, SettingsChanged};
use crate::ui::components::{Slider, TextInput, ToggleState};
use crate::ui::widgets::*;
use bevy::app::AppExit;
use bevy::color::Color;
use bevy::prelude::*;
use planetgen::constants::*;
use rand::Rng;

pub fn setup_world_generation_menu(
    mut commands: Commands,
    settings: Res<PlanetGenerationSettings>,
) {
    // Root container node
    let root_node = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        ..default()
    };

    // Main 3D view area node (left side)
    let main_area_node = Node {
        width: Val::Percent(75.0),
        height: Val::Percent(100.0),
        display: Display::Flex,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    };

    // Placeholder text for when no planet exists
    let placeholder_text = Text::new("Press Generate to see the preview");
    let placeholder_font = TextFont {
        font_size: 32.0,
        ..default()
    };
    let placeholder_color = TextColor(Color::srgba(1.0, 1.0, 1.0, 0.6));

    // Settings panel node (right side)
    let settings_panel_node = Node {
        width: Val::Percent(25.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(Val::Px(20.0)),
        row_gap: Val::Px(15.0),
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Stretch,
        ..default()
    };

    // Title text and styling
    let title_text = Text::new("Planet Settings");
    let title_font = TextFont {
        font_size: 24.0,
        ..default()
    };
    let title_node = Node {
        margin: UiRect::bottom(Val::Px(20.0)),
        align_self: AlignSelf::Center,
        ..default()
    };

    // Seed section container
    let seed_section_node = Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(5.0),
        width: Val::Percent(100.0),
        ..default()
    };

    // Seed label
    let seed_label_text = Text::new("Seed");
    let seed_label_font = TextFont {
        font_size: 16.0,
        ..default()
    };

    // Seed input row container
    let seed_row_node = Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(5.0),
        width: Val::Percent(100.0),
        ..default()
    };

    // Seed text input field
    let seed_input_node = Node {
        width: Val::Px(140.0),
        height: Val::Px(30.0),
        padding: UiRect::all(Val::Px(8.0)),
        border: UiRect::all(Val::Px(1.0)),
        ..default()
    };

    // Seed input text
    let seed_text = Text::new(&settings.seed.to_string());
    let seed_text_font = TextFont {
        font_size: 14.0,
        ..default()
    };

    // Spacer node
    let spacer_node = Node {
        height: Val::Px(20.0),
        ..default()
    };

    // Create a side panel layout instead of full screen
    commands
        .spawn((root_node, WorldGenerationMenu))
        .with_children(|parent| {
            // Main 3D view area (left side)
            parent
                .spawn((
                    main_area_node,
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.1)), // Dark background
                    MainArea,
                ))
                .with_children(|parent| {
                    // Placeholder text shown when no planet exists
                    parent.spawn((
                        placeholder_text,
                        placeholder_font,
                        placeholder_color,
                        PlaceholderText,
                    ));
                });

            // Settings panel (right side)
            parent
                .spawn((
                    settings_panel_node,
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)), // Semi-transparent dark background
                ))
                .with_children(|parent| {
                    // Title
                    parent.spawn((title_text, title_font, TextColor(Color::WHITE), title_node));

                    // Seed input section
                    parent.spawn(seed_section_node).with_children(|parent| {
                        // Seed label
                        parent.spawn((seed_label_text, seed_label_font, TextColor(Color::WHITE)));

                        // Seed input row with text field and random button
                        parent.spawn(seed_row_node).with_children(|parent| {
                            // Text input field for seed
                            parent
                                .spawn((
                                    seed_input_node,
                                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                                    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                                    TextInput::new(settings.seed.to_string()),
                                    SeedInput,
                                    Interaction::default(),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        seed_text,
                                        seed_text_font,
                                        TextColor(Color::WHITE),
                                    ));
                                });

                            // Random seed button
                            spawn_button_with_marker(
                                parent,
                                "RND",
                                55.0,
                                30.0,
                                Color::srgb(0.3, 0.5, 0.7),
                                Color::srgb(0.4, 0.6, 0.8),
                                Color::srgb(0.2, 0.4, 0.6),
                                0.0,
                                RandomSeedButton,
                            );
                        });
                    });

                    // Planet Radius Slider
                    spawn_slider_with_marker(
                        parent,
                        "Planet Radius",
                        settings.radius,
                        PLANET_MIN_RADIUS,
                        PLANET_MAX_RADIUS,
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
                    parent.spawn(spacer_node);

                    // Generate Planet button
                    spawn_default_button_with_marker(
                        parent,
                        "Generate Planet",
                        Color::srgb(0.2, 0.7, 0.2),
                        Color::srgb(0.3, 0.8, 0.3),
                        Color::srgb(0.1, 0.6, 0.1),
                        GeneratePlanetButton,
                    );

                    // Quit button
                    spawn_default_button_with_marker(
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

pub fn cleanup_world_generation_menu(
    mut commands: Commands,
    query: Query<Entity, With<WorldGenerationMenu>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_buttons(
    generate_query: Query<&Interaction, (Changed<Interaction>, With<GeneratePlanetButton>)>,
    quit_query: Query<&Interaction, (Changed<Interaction>, With<QuitButton>)>,
    random_seed_query: Query<&Interaction, (Changed<Interaction>, With<RandomSeedButton>)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut planet_generation_events: EventWriter<GeneratePlanetEvent>,
    mut settings_changed_events: EventWriter<SettingsChanged>,
    mut seed_input_query: Query<(&mut TextInput, &Children), With<SeedInput>>,
    mut text_query: Query<&mut Text>,
) {
    // Handle Generate Planet button
    for interaction in &generate_query {
        if *interaction == Interaction::Pressed {
            // Send event to generate planet instead of changing state
            planet_generation_events.write(GeneratePlanetEvent);
        }
    }

    // Handle Random Seed button
    for interaction in &random_seed_query {
        if *interaction == Interaction::Pressed {
            let new_seed = rand::rng().random::<u64>();

            // Update the text input with new random seed
            for (mut text_input, children) in seed_input_query.iter_mut() {
                text_input.text = new_seed.to_string();

                // Update the displayed text
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = new_seed.to_string();
                    }
                }
            }

            settings_changed_events.write(SettingsChanged);
        }
    }

    // Handle Quit button
    for interaction in &quit_query {
        if *interaction == Interaction::Pressed {
            app_exit_events.write(AppExit::Success);
        }
    }
}

pub fn detect_settings_changes(
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

pub fn update_settings_on_change(
    mut settings_changed_events: EventReader<SettingsChanged>,
    mut settings: ResMut<PlanetGenerationSettings>,
    radius_slider_query: Query<&Slider, With<RadiusSlider>>,
    cells_slider_query: Query<&Slider, With<CellsPerUnitSlider>>,
    plates_slider_query: Query<&Slider, With<NumPlatesSlider>>,
    micro_plates_slider_query: Query<&Slider, With<NumMicroPlatesSlider>>,
    toggle_query: Query<&ToggleState, With<ShowArrowsToggle>>,
    seed_input_query: Query<&TextInput, With<SeedInput>>,
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

        // Update seed from text input
        for text_input in &seed_input_query {
            if let Ok(parsed_seed) = text_input.text.parse::<u64>() {
                settings.seed = parsed_seed;
            }
        }
    }
}

pub fn update_main_area_content(
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

pub fn handle_arrow_toggle_change(
    mut toggle_arrows_events: EventWriter<ToggleArrowsEvent>,
    toggle_query: Query<&ToggleState, (With<ShowArrowsToggle>, Changed<ToggleState>)>,
) {
    // Send arrow toggle event immediately when the toggle changes
    for toggle_state in &toggle_query {
        toggle_arrows_events.write(ToggleArrowsEvent {
            show_arrows: toggle_state.is_on,
        });
    }
}

pub fn handle_seed_input(
    mut seed_input_query: Query<
        (&mut TextInput, &Children, &Interaction),
        (With<SeedInput>, Changed<Interaction>),
    >,
    mut text_query: Query<&mut Text>,
    mut settings_changed_events: EventWriter<SettingsChanged>,
    keys: Res<ButtonInput<KeyCode>>,
    mut char_events: EventReader<ReceivedCharacter>,
) {
    // Handle text input focus
    for (mut text_input, children, interaction) in seed_input_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            text_input.is_focused = true;
            text_input.cursor_position = text_input.text.len();
        }
    }

    // Handle keyboard input for focused text field
    for (mut text_input, children, _) in seed_input_query.iter_mut() {
        if !text_input.is_focused {
            continue;
        }

        let mut text_changed = false;

        // Handle backspace
        if keys.just_pressed(KeyCode::Backspace) && !text_input.text.is_empty() {
            text_input.text.pop();
            if text_input.cursor_position > 0 {
                text_input.cursor_position -= 1;
            }
            text_changed = true;
        }

        // Handle enter (confirm input and lose focus)
        if keys.just_pressed(KeyCode::Enter) {
            text_input.is_focused = false;
            text_changed = true;
        }

        // Handle escape (lose focus)
        if keys.just_pressed(KeyCode::Escape) {
            text_input.is_focused = false;
        }

        // Handle character input
        for char_event in char_events.read() {
            let ch = char_event.char;
            if ch.is_ascii_digit() && text_input.text.len() < 20 {
                // Only allow digits and limit length for u64
                text_input.text.push(ch);
                text_input.cursor_position += 1;
                text_changed = true;
            }
        }

        // Update displayed text if changed
        if text_changed {
            for child in children.iter() {
                if let Ok(mut text) = text_query.get_mut(child) {
                    **text = text_input.text.clone();
                }
            }
            settings_changed_events.write(SettingsChanged);
        }
    }

    // Lose focus when clicking elsewhere
    for (mut text_input, _, interaction) in seed_input_query.iter_mut() {
        if text_input.is_focused && *interaction == Interaction::None {
            text_input.is_focused = false;
        }
    }
}

pub fn generate_char_events(
    keys: Res<ButtonInput<KeyCode>>,
    mut char_events: EventWriter<ReceivedCharacter>,
) {
    // Generate character events from keyboard input
    let digits = [
        (KeyCode::Digit0, '0'),
        (KeyCode::Digit1, '1'),
        (KeyCode::Digit2, '2'),
        (KeyCode::Digit3, '3'),
        (KeyCode::Digit4, '4'),
        (KeyCode::Digit5, '5'),
        (KeyCode::Digit6, '6'),
        (KeyCode::Digit7, '7'),
        (KeyCode::Digit8, '8'),
        (KeyCode::Digit9, '9'),
    ];

    for (key, ch) in digits {
        if keys.just_pressed(key) {
            char_events.write(ReceivedCharacter { char: ch });
        }
    }
}
