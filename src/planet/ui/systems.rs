use crate::planet::components::CameraRotationMode;
use crate::planet::events::*;
use crate::planet::resources::PlanetGenerationSettings;
use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

#[derive(Resource, Default, Clone, PartialEq)]
pub enum ViewTab {
    #[default]
    Continent,
    Tectonic,
    Wind,
    Temperature,
    Precipitations,
}

pub fn setup_world_generation_menu(mut commands: Commands) {
    commands.init_resource::<ViewTab>();
    commands.init_resource::<CameraRotationMode>();
}

pub fn cleanup_world_generation_menu(mut commands: Commands) {
    commands.remove_resource::<ViewTab>();
    commands.remove_resource::<CameraRotationMode>();
}

pub fn render_planet_generation_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<PlanetGenerationSettings>,
    mut view_tab: ResMut<ViewTab>,
    mut planet_generation_events: MessageWriter<GeneratePlanetEvent>,
    mut generate_new_seed_events: MessageWriter<GenerateNewSeedEvent>,
    mut tab_switch_events: MessageWriter<TabSwitchEvent>,
    mut wind_tab_events: MessageWriter<WindTabActiveEvent>,
    mut temperature_tab_events: MessageWriter<TemperatureTabActiveEvent>,
    mut precipitation_tab_events: MessageWriter<PrecipitationTabActiveEvent>,
    mut app_exit_events: MessageWriter<AppExit>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::SidePanel::right("settings_panel")
        .default_width(350.0)
        .resizable(true)
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Planet Settings");
                ui.add_space(10.0);

                // View tabs
                ui.horizontal(|ui| {
                    let mut tab_changed = false;
                    let old_tab = (*view_tab).clone();

                    if ui
                        .selectable_label(*view_tab == ViewTab::Continent, "Continent")
                        .clicked()
                    {
                        *view_tab = ViewTab::Continent;
                        tab_changed = old_tab != *view_tab;
                    }
                    if ui
                        .selectable_label(*view_tab == ViewTab::Tectonic, "Tectonic")
                        .clicked()
                    {
                        *view_tab = ViewTab::Tectonic;
                        tab_changed = old_tab != *view_tab;
                    }
                    if ui
                        .selectable_label(*view_tab == ViewTab::Wind, "Wind")
                        .clicked()
                    {
                        *view_tab = ViewTab::Wind;
                        tab_changed = old_tab != *view_tab;
                    }
                    if ui
                        .selectable_label(*view_tab == ViewTab::Temperature, "Temperature")
                        .clicked()
                    {
                        *view_tab = ViewTab::Temperature;
                        tab_changed = old_tab != *view_tab;
                    }
                    if ui
                        .selectable_label(*view_tab == ViewTab::Precipitations, "Precipitations")
                        .clicked()
                    {
                        *view_tab = ViewTab::Precipitations;
                        tab_changed = old_tab != *view_tab;
                    }

                    // Update visibility when tab changes
                    if tab_changed {
                        // Emit ONE event with the target tab
                        let tab_type = match *view_tab {
                            ViewTab::Continent => ViewTabType::Continent,
                            ViewTab::Tectonic => ViewTabType::Tectonic,
                            ViewTab::Wind => ViewTabType::Wind,
                            ViewTab::Temperature => ViewTabType::Temperature,
                            ViewTab::Precipitations => ViewTabType::Precipitations,
                        };

                        tab_switch_events.write(TabSwitchEvent { tab: tab_type });

                        // Emit wind event for particle spawning/despawning
                        let is_wind = *view_tab == ViewTab::Wind;
                        wind_tab_events.write(WindTabActiveEvent { active: is_wind });

                        // Emit temperature event for mesh generation
                        let is_temperature = *view_tab == ViewTab::Temperature;
                        temperature_tab_events.write(TemperatureTabActiveEvent {
                            active: is_temperature,
                        });

                        // Emit precipitation event for mesh generation
                        let is_precipitation = *view_tab == ViewTab::Precipitations;
                        precipitation_tab_events.write(PrecipitationTabActiveEvent {
                            active: is_precipitation,
                        });
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Tab-specific content
                match *view_tab {
                    ViewTab::Continent => {
                        // Continent tab content
                        render_continent_tab(
                            ui,
                            &mut settings,
                            &mut generate_new_seed_events,
                            &mut planet_generation_events,
                        );
                    }
                    ViewTab::Tectonic => {
                        // Tectonic tab content
                        render_tectonic_tab(ui, &mut settings);
                    }
                    ViewTab::Wind => {
                        // Wind tab content
                        render_wind_tab(ui, &mut settings);
                    }
                    ViewTab::Temperature => {
                        // Temperature tab content
                        render_temperature_tab(ui, &mut settings);
                    }
                    ViewTab::Precipitations => {
                        // Precipitations tab content
                        render_precipitation_tab(ui, &mut settings);
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                if ui.button("Quit").clicked() {
                    app_exit_events.write(AppExit::Success);
                }
            });
        });
}

fn render_continent_tab(
    ui: &mut egui::Ui,
    settings: &mut PlanetGenerationSettings,
    generate_new_seed_events: &mut MessageWriter<GenerateNewSeedEvent>,
    planet_generation_events: &mut MessageWriter<GeneratePlanetEvent>,
) {
    // Seed section
    ui.heading("General");
    ui.add_space(5.0);

    ui.label("Seed");
    ui.horizontal(|ui| {
        ui.label(settings.user_seed.to_string());
        if ui.button("Random").clicked() {
            generate_new_seed_events.write(GenerateNewSeedEvent);
        }
    });
    ui.add_space(10.0);

    ui.separator();
    ui.add_space(10.0);

    // Continent generation settings
    ui.heading("Continent Generation");
    ui.add_space(5.0);

    ui.label("Continent Distortion Frequency");
    ui.add(egui::Slider::new(&mut settings.distortion_frequency, 1.0..=10.0).step_by(0.1));

    ui.label("Continent Distortion Strength");
    ui.add(egui::Slider::new(&mut settings.distortion_amplitude, 0.0..=1.0).step_by(0.01));

    ui.label("Ocean Coverage");
    ui.add(egui::Slider::new(&mut settings.continent_threshold, -1.0..=1.0).step_by(0.01));

    ui.label("Continent Shore Distortion Frequency");
    ui.add(egui::Slider::new(&mut settings.detail_frequency, 5.0..=20.0).step_by(0.1));

    ui.label("Continent Shore Distortion Scale");
    ui.add(egui::Slider::new(&mut settings.detail_amplitude, 0.05..=0.5).step_by(0.01));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Mountain settings
    ui.heading("Mountain Settings");
    ui.add_space(5.0);

    ui.label("Mountain Snow Threshold");
    ui.add(egui::Slider::new(&mut settings.snow_threshold, 0.5..=4.0).step_by(0.01));

    ui.label("Mountain Height");
    ui.add(egui::Slider::new(&mut settings.mountain_height, 2.0..=5.0).step_by(0.01));

    ui.label("Mountain Width");
    ui.add(egui::Slider::new(&mut settings.mountain_width, 0.03..=0.25).step_by(0.001));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Generate Planet button (only on Continent tab)
    if ui.button("Generate Planet").clicked() {
        planet_generation_events.write(GeneratePlanetEvent);
    }
}

fn render_tectonic_tab(ui: &mut egui::Ui, settings: &mut PlanetGenerationSettings) {
    ui.heading("Tectonic Plate Settings");
    ui.add_space(5.0);

    ui.label("Number of Major Plates");
    ui.add(egui::Slider::new(&mut settings.num_plates, 3..=15));

    ui.label("Number of Micro Plates");
    ui.add(egui::Slider::new(&mut settings.num_micro_plates, 0..=20));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Plate Boundary Flow");
    ui.add_space(5.0);

    ui.label("Flow Warp Frequency");
    ui.add(egui::Slider::new(&mut settings.flow_warp_freq, 0.1..=2.0).step_by(0.05));

    ui.label("Flow Warp Steps");
    ui.add(egui::Slider::new(&mut settings.flow_warp_steps, 1..=8));

    ui.label("Flow Step Angle");
    ui.add(egui::Slider::new(&mut settings.flow_warp_step_angle, 0.01..=0.5).step_by(0.01));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Visualization");
    ui.add_space(5.0);

    ui.checkbox(&mut settings.show_arrows, "Show Plate Direction Arrows");
}

fn render_wind_tab(ui: &mut egui::Ui, settings: &mut PlanetGenerationSettings) {
    ui.add_space(5.0);

    ui.heading("Wind Speed");
    ui.add_space(5.0);

    ui.label("Zonal Speed (East/West)");
    ui.add(egui::Slider::new(&mut settings.wind_zonal_speed, 0.0..=10.0).step_by(0.1));

    ui.add_space(10.0);

    ui.separator();
    ui.add_space(10.0);

    ui.heading("Particle Settings");
    ui.add_space(5.0);

    // Display current particle count (read-only, set via config)
    ui.label(format!(
        "Particle Count: {} (set in config)",
        settings.wind_particle_count
    ));

    ui.add_space(10.0);

    ui.label("Particle Lifespan (seconds)");
    ui.add(egui::Slider::new(&mut settings.wind_particle_lifespan, 1.0..=10.0).step_by(0.1));
    ui.label("Lower lifespan = faster respawn rate");
    ui.add_space(5.0);

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Vertical Air Movement");
    ui.add_space(5.0);

    ui.checkbox(&mut settings.show_vertical_air, "Show Vertical Air Movement");

    ui.add_space(5.0);
    ui.label("Color Scale:");
    ui.horizontal(|ui| {
        ui.label("Blue: Rising air (convergence)");
    });
    ui.horizontal(|ui| {
        ui.label("White: Neutral");
    });
    ui.horizontal(|ui| {
        ui.label("Red: Sinking air (divergence)");
    });

    ui.separator();
    ui.add_space(10.0);

    ui.heading("Wind Deflection");
    ui.add_space(5.0);

    ui.label("Height Threshold");
    ui.add(egui::Slider::new(&mut settings.wind_deflection_height_threshold, 0.0..=1.0).step_by(0.01));

    ui.label("Height Scale");
    ui.add(egui::Slider::new(&mut settings.wind_deflection_height_scale, 0.5..=5.0).step_by(0.1));

    ui.label("Spread Radius");
    ui.add(egui::Slider::new(&mut settings.wind_deflection_spread_radius, 1..=8));

    ui.label("Spread Decay");
    ui.add(egui::Slider::new(&mut settings.wind_deflection_spread_decay, 0.1..=0.9).step_by(0.01));

    ui.label("Deflection Strength");
    ui.add(egui::Slider::new(&mut settings.wind_deflection_strength, 0.0..=1.0).step_by(0.01));

    ui.label("Deflection Iterations");
    ui.add(egui::Slider::new(&mut settings.wind_deflection_iterations, 1..=10));

    ui.add_space(5.0);
}

fn render_temperature_tab(ui: &mut egui::Ui, settings: &mut PlanetGenerationSettings) {
    ui.add_space(5.0);

    ui.heading("Temperature Generation");
    ui.add_space(5.0);

    ui.label("Equator Temperature (°C)");
    ui.add(egui::Slider::new(&mut settings.temperature_equator_temp, 20.0..=50.0).step_by(1.0));
    ui.label("Hottest temperature at the equator");

    ui.add_space(5.0);

    ui.label("Pole Temperature (°C)");
    ui.add(egui::Slider::new(&mut settings.temperature_pole_temp, -50.0..=-10.0).step_by(1.0));
    ui.label("Coldest temperature at the poles");

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Color Scale Range");
    ui.add_space(5.0);

    ui.label("Maximum Temperature (°C)");
    ui.add(egui::Slider::new(&mut settings.temperature_max_temp, 30.0..=100.0).step_by(5.0));
    ui.label("Red end of color gradient");

    ui.add_space(5.0);

    ui.label("Minimum Temperature (°C)");
    ui.add(egui::Slider::new(&mut settings.temperature_min_temp, -100.0..=-20.0).step_by(5.0));
    ui.label("Blue end of color gradient");

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Temperature Settings");
    ui.add_space(5.0);

    ui.label("Land Temperature Bonus");
    ui.add(
        egui::Slider::new(&mut settings.land_temperature_bonus, 0.0..=20.0)
            .step_by(0.5)
            .suffix("°C"),
    );
    ui.label("Extra warmth for land above sea level");

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Temperature Map");
    ui.add_space(5.0);

    ui.label("Displaying latitude-based temperature distribution:");
    ui.add_space(10.0);

    // Color legend showing the actual range
    ui.label("Color Scale:");
    ui.horizontal(|ui| {
        ui.label("🔵 Light Blue:");
        ui.label(format!("{:.0}°C", settings.temperature_min_temp));
    });
    ui.horizontal(|ui| {
        ui.label("🟦 Cyan:");
        ui.label(format!(
            "{:.0}°C",
            settings.temperature_min_temp * 0.8 + settings.temperature_max_temp * 0.2
        ));
    });
    ui.horizontal(|ui| {
        ui.label("🟢 Green:");
        ui.label(format!(
            "{:.0}°C",
            settings.temperature_min_temp * 0.6 + settings.temperature_max_temp * 0.4
        ));
    });
    ui.horizontal(|ui| {
        ui.label("🟡 Yellow:");
        ui.label(format!(
            "{:.0}°C",
            settings.temperature_min_temp * 0.4 + settings.temperature_max_temp * 0.6
        ));
    });
    ui.horizontal(|ui| {
        ui.label("🟠 Orange:");
        ui.label(format!(
            "{:.0}°C",
            settings.temperature_min_temp * 0.2 + settings.temperature_max_temp * 0.8
        ));
    });
    ui.horizontal(|ui| {
        ui.label("🔴 Red:");
        ui.label(format!("{:.0}°C", settings.temperature_max_temp));
    });
}

fn render_precipitation_tab(ui: &mut egui::Ui, _settings: &mut PlanetGenerationSettings) {
    ui.add_space(5.0);

    ui.label("Color Scale:");
    ui.horizontal(|ui| {
        ui.label("Tan/Yellow: Dry (0%)");
    });
    ui.horizontal(|ui| {
        ui.label("Green: Moderate (50%)");
    });
    ui.horizontal(|ui| {
        ui.label("Blue: Wet (100%)");
    });
}
