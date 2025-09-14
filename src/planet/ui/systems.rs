use crate::planet::components::PlanetEntity;
use crate::planet::events::*;
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::ui::components::*;
use crate::ui::components::{Slider, ToggleState};
use crate::ui::widgets::*;
use bevy::app::AppExit;
use bevy::color::Color;
use bevy::prelude::*;

pub fn setup_world_generation_menu(
    mut commands: Commands,
    settings: Res<PlanetGenerationSettings>,
) {
    let config = planetgen::get_config();
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
                            // label for seed, can be replaced with bevy_simple_text_input lib
                            parent.spawn((
                                Text::new(&settings.user_seed.to_string()),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                                SeedDisplay,
                            ));
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
                        config.generation.planet_min_radius,
                        config.generation.planet_max_radius,
                        false,
                        200.0,
                        RadiusSlider,
                    );

                    // Number of Plates Slider
                    spawn_slider_with_marker(
                        parent,
                        "Number of Plates",
                        settings.num_plates as f32,
                        3.0,
                        20.0,
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

                    // I used this code to conveniently determine good coefficients for plate
                    // boundary distortion, for now they're constant, but I want to keep
                    // the code in case I need more tweaking in the future
                    // // Flow Warp Frequency Slider
                    // spawn_slider_with_marker(
                    //     parent,
                    //     "Flow Warp Frequency",
                    //     settings.flow_warp_freq,
                    //     0.1,
                    //     1.0,
                    //     false,
                    //     200.0,
                    //     FlowWarpFreqSlider,
                    // );
                    //
                    // // Flow Warp Steps Slider
                    // spawn_slider_with_marker(
                    //     parent,
                    //     "Flow Warp Steps",
                    //     settings.flow_warp_steps as f32,
                    //     1.0,
                    //     10.0,
                    //     true,
                    //     200.0,
                    //     FlowWarpStepsSlider,
                    // );
                    //
                    // // Flow Warp Step Angle Slider
                    // spawn_slider_with_marker(
                    //     parent,
                    //     "Flow Warp Step Angle",
                    //     settings.flow_warp_step_angle,
                    //     0.05,
                    //     0.5,
                    //     false,
                    //     200.0,
                    //     FlowWarpStepAngleSlider,
                    // );

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
    mut generate_new_seed_events: EventWriter<GenerateNewSeedEvent>,
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
            generate_new_seed_events.write(GenerateNewSeedEvent);
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
    plates_slider_query: Query<&Slider, (With<NumPlatesSlider>, Changed<Slider>)>,
    micro_plates_slider_query: Query<&Slider, (With<NumMicroPlatesSlider>, Changed<Slider>)>,
    // flow_freq_slider_query: Query<&Slider, (With<FlowWarpFreqSlider>, Changed<Slider>)>,
    // flow_amp_slider_query: Query<&Slider, (With<FlowWarpAmpSlider>, Changed<Slider>)>,
    // flow_steps_slider_query: Query<&Slider, (With<FlowWarpStepsSlider>, Changed<Slider>)>,
    // flow_angle_slider_query: Query<&Slider, (With<FlowWarpStepAngleSlider>, Changed<Slider>)>,
    toggle_query: Query<&ToggleState, (With<ShowArrowsToggle>, Changed<ToggleState>)>,
) {
    // Check if any slider or toggle has changed and send event
    let has_changes = !radius_slider_query.is_empty()
        || !plates_slider_query.is_empty()
        || !micro_plates_slider_query.is_empty()
        // || !flow_freq_slider_query.is_empty()
        // || !flow_amp_slider_query.is_empty()
        // || !flow_steps_slider_query.is_empty()
        // || !flow_angle_slider_query.is_empty()
        || !toggle_query.is_empty();

    if has_changes {
        settings_changed_events.write(SettingsChanged);
    }
}

pub fn update_settings_on_change(
    mut settings_changed_events: EventReader<SettingsChanged>,
    mut settings: ResMut<PlanetGenerationSettings>,
    radius_slider_query: Query<&Slider, With<RadiusSlider>>,
    plates_slider_query: Query<&Slider, With<NumPlatesSlider>>,
    micro_plates_slider_query: Query<&Slider, With<NumMicroPlatesSlider>>,
    // flow_freq_slider_query: Query<&Slider, With<FlowWarpFreqSlider>>,
    // flow_steps_slider_query: Query<&Slider, With<FlowWarpStepsSlider>>,
    // flow_angle_slider_query: Query<&Slider, With<FlowWarpStepAngleSlider>>,
    toggle_query: Query<&ToggleState, With<ShowArrowsToggle>>,
) {
    // Only update settings if we received a change event
    for _ in settings_changed_events.read() {
        // Update settings from current slider and toggle values
        for slider in &radius_slider_query {
            settings.radius = slider.current_value;
        }
        for slider in &plates_slider_query {
            settings.num_plates = slider.current_value as usize;
        }
        for slider in &micro_plates_slider_query {
            settings.num_micro_plates = slider.current_value as usize;
        }
        // for slider in &flow_freq_slider_query {
        //     settings.flow_warp_freq = slider.current_value;
        // }
        // for slider in &flow_steps_slider_query {
        //     settings.flow_warp_steps = slider.current_value as usize;
        // }
        // for slider in &flow_angle_slider_query {
        //     settings.flow_warp_step_angle = slider.current_value;
        // }
        for toggle_state in &toggle_query {
            settings.show_arrows = toggle_state.is_on;
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

pub fn update_seed_display_on_change(
    settings: Res<PlanetGenerationSettings>,
    mut seed_display_query: Query<&mut Text, With<SeedDisplay>>,
) {
    if settings.is_changed() {
        for mut text in seed_display_query.iter_mut() {
            **text = settings.user_seed.to_string();
        }
    }
}
