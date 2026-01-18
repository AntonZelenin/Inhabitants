use crate::planet::components::{ContinentViewMesh, PlateViewMesh};
use crate::planet::events::*;
use crate::planet::resources::PlanetGenerationSettings;
use bevy::app::AppExit;
use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub fn setup_world_generation_menu(
    // This function is now empty - egui renders everything dynamically
) {
}

pub fn cleanup_world_generation_menu(
    // egui doesn't need cleanup
) {
}

pub fn render_planet_generation_ui(
    mut contexts: EguiContexts,
    mut settings: ResMut<PlanetGenerationSettings>,
    mut planet_generation_events: MessageWriter<GeneratePlanetEvent>,
    mut generate_new_seed_events: MessageWriter<GenerateNewSeedEvent>,
    mut app_exit_events: MessageWriter<AppExit>,
    mut continent_mesh_query: Query<&mut Visibility, (With<ContinentViewMesh>, Without<PlateViewMesh>)>,
    mut plate_mesh_query: Query<&mut Visibility, (With<PlateViewMesh>, Without<ContinentViewMesh>)>,
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

                // Seed section
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

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                // View options
                ui.heading("View Options");
                ui.add_space(5.0);

                let mut view_mode_plates = settings.view_mode_plates;
                if ui.checkbox(&mut view_mode_plates, "View Tectonic Plates").changed() {
                    settings.view_mode_plates = view_mode_plates;

                    // Update mesh visibility immediately
                    for mut visibility in continent_mesh_query.iter_mut() {
                        *visibility = if view_mode_plates {
                            Visibility::Hidden
                        } else {
                            Visibility::Visible
                        };
                    }

                    for mut visibility in plate_mesh_query.iter_mut() {
                        *visibility = if view_mode_plates {
                            Visibility::Visible
                        } else {
                            Visibility::Hidden
                        };
                    }
                }

                ui.add_space(20.0);
                ui.separator();
                ui.add_space(10.0);

                // Action buttons
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
