use crate::planet::components::{ContinentView, TectonicPlateView};
use crate::planet::events::*;
use crate::planet::resources::PlanetGenerationSettings;
use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Resource, Default, Clone, PartialEq)]
pub enum ViewTab {
    #[default]
    Continent,
    Tectonic,
    Wind,
}

pub fn setup_world_generation_menu(
    mut commands: Commands,
) {
    commands.init_resource::<ViewTab>();
}

pub fn cleanup_world_generation_menu(
    mut commands: Commands,
) {
    commands.remove_resource::<ViewTab>();
}

pub fn render_planet_generation_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<PlanetGenerationSettings>,
    mut view_tab: ResMut<ViewTab>,
    mut planet_generation_events: MessageWriter<GeneratePlanetEvent>,
    mut generate_new_seed_events: MessageWriter<GenerateNewSeedEvent>,
    mut wind_tab_events: MessageWriter<WindTabActiveEvent>,
    mut app_exit_events: MessageWriter<AppExit>,
    mut continent_view_query: Query<&mut Visibility, (With<ContinentView>, Without<TectonicPlateView>)>,
    mut plate_view_query: Query<&mut Visibility, (With<TectonicPlateView>, Without<ContinentView>)>,
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

                    if ui.selectable_label(*view_tab == ViewTab::Continent, "Continent").clicked() {
                        *view_tab = ViewTab::Continent;
                        tab_changed = old_tab != *view_tab;
                    }
                    if ui.selectable_label(*view_tab == ViewTab::Tectonic, "Tectonic").clicked() {
                        *view_tab = ViewTab::Tectonic;
                        tab_changed = old_tab != *view_tab;
                    }
                    if ui.selectable_label(*view_tab == ViewTab::Wind, "Wind").clicked() {
                        *view_tab = ViewTab::Wind;
                        tab_changed = old_tab != *view_tab;
                    }

                    // Update visibility when tab changes
                    if tab_changed {
                        let show_plates = *view_tab == ViewTab::Tectonic;
                        settings.view_mode_plates = show_plates;

                        // Emit wind tab event when switching to/from wind tab
                        let is_wind_active = *view_tab == ViewTab::Wind;
                        wind_tab_events.write(WindTabActiveEvent { active: is_wind_active });

                        // Hide/show all entities in continent view
                        for mut visibility in continent_view_query.iter_mut() {
                            *visibility = if show_plates {
                                Visibility::Hidden
                            } else {
                                Visibility::Visible
                            };
                        }

                        // Hide/show all entities in tectonic plate view
                        for mut visibility in plate_view_query.iter_mut() {
                            *visibility = if show_plates {
                                Visibility::Visible
                            } else {
                                Visibility::Hidden
                            };
                        }
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // Tab-specific content
                match *view_tab {
                    ViewTab::Continent => {
                        // Continent tab content
                        render_continent_tab(ui, &mut settings, &mut generate_new_seed_events);
                    }
                    ViewTab::Tectonic => {
                        // Tectonic tab content
                        render_tectonic_tab(ui, &mut settings);
                    }
                    ViewTab::Wind => {
                        // Wind tab content
                        render_wind_tab(ui, &mut settings);
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                // Common action buttons
                if ui.button("Generate Planet").clicked() {
                    planet_generation_events.write(GeneratePlanetEvent);
                }

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
    ui.add(egui::Slider::new(&mut settings.distortion_frequency, 1.0..=10.0)
        .step_by(0.1));

    ui.label("Continent Distortion Strength");
    ui.add(egui::Slider::new(&mut settings.distortion_amplitude, 0.0..=1.0)
        .step_by(0.01));

    ui.label("Ocean Coverage");
    ui.add(egui::Slider::new(&mut settings.continent_threshold, -1.0..=1.0)
        .step_by(0.01));

    ui.label("Continent Shore Distortion Frequency");
    ui.add(egui::Slider::new(&mut settings.detail_frequency, 5.0..=20.0)
        .step_by(0.1));

    ui.label("Continent Shore Distortion Scale");
    ui.add(egui::Slider::new(&mut settings.detail_amplitude, 0.05..=0.5)
        .step_by(0.01));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    // Mountain settings
    ui.heading("Mountain Settings");
    ui.add_space(5.0);

    ui.label("Mountain Snow Threshold");
    ui.add(egui::Slider::new(&mut settings.snow_threshold, 0.5..=4.0)
        .step_by(0.01));

    ui.label("Mountain Height");
    ui.add(egui::Slider::new(&mut settings.mountain_height, 2.0..=5.0)
        .step_by(0.01));

    ui.label("Mountain Width");
    ui.add(egui::Slider::new(&mut settings.mountain_width, 0.03..=0.25)
        .step_by(0.001));
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
    ui.add(egui::Slider::new(&mut settings.flow_warp_freq, 0.1..=2.0)
        .step_by(0.05));

    ui.label("Flow Warp Steps");
    ui.add(egui::Slider::new(&mut settings.flow_warp_steps, 1..=8));

    ui.label("Flow Step Angle");
    ui.add(egui::Slider::new(&mut settings.flow_warp_step_angle, 0.01..=0.5)
        .step_by(0.01));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.heading("Visualization");
    ui.add_space(5.0);

    ui.checkbox(&mut settings.show_arrows, "Show Plate Direction Arrows");
}

fn render_wind_tab(ui: &mut egui::Ui, settings: &mut PlanetGenerationSettings) {
    ui.add_space(5.0);

    ui.add_space(10.0);

    ui.separator();
    ui.add_space(10.0);

    ui.heading("Particle Settings");
    ui.add_space(5.0);

    // Display current particle count (read-only, set via config)
    ui.label(format!("Particle Count: {} (set in config)", settings.wind_particle_count));
    ui.add_space(5.0);

    ui.label("Wind Speed");
    ui.add(egui::Slider::new(&mut settings.wind_speed, 0.1..=2.0)
        .step_by(0.1));

    ui.label("Trail Length");
    ui.add(egui::Slider::new(&mut settings.wind_trail_length, 0.5..=5.0)
        .step_by(0.1));

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);

    ui.colored_label(
        egui::Color32::GRAY,
        "Currently showing uniform eastward wind flow.\nFuture updates will add realistic atmospheric circulation."
    );
}
