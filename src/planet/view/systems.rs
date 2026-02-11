use crate::planet::components::{ContinentView, OceanEntity, TectonicPlateView};
use crate::planet::events::{TabSwitchEvent, ViewTabType};
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::temperature::systems::TemperatureMesh;
use crate::planet::precipitation::systems::PrecipitationMesh;
use crate::planet::wind::systems::VerticalAirMesh;
use bevy::prelude::*;

/// CENTRALIZED tab visibility handler - handles ALL tab switching in ONE place
/// Wind particles are managed by their own systems (handle_wind_tab_events + spawn_debug_particles)
pub fn handle_tab_visibility(
    mut tab_switch_events: MessageReader<TabSwitchEvent>,
    planet_settings: Res<PlanetGenerationSettings>,
    continent_view_query: Query<Entity, With<ContinentView>>,
    ocean_query: Query<Entity, With<OceanEntity>>,
    plate_view_query: Query<Entity, With<TectonicPlateView>>,
    temperature_mesh_query: Query<Entity, With<TemperatureMesh>>,
    precipitation_mesh_query: Query<Entity, With<PrecipitationMesh>>,
    vertical_air_query: Query<Entity, With<VerticalAirMesh>>,
    mut commands: Commands,
) {
    for event in tab_switch_events.read() {
        info!("Switching to {:?} tab - handling ALL visibility", event.tab);

        match event.tab {
            ViewTabType::Continent => {
                // Show: Continent mesh + Ocean
                // Hide: Tectonic plates, Temperature meshes, Precipitation meshes, Vertical air

                for entity in continent_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }

                for entity in ocean_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }

                for entity in plate_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in temperature_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in precipitation_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in vertical_air_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }

            ViewTabType::Wind => {
                // Wind particles are managed by handle_wind_tab_events + spawn_debug_particles
                // If vertical air overlay is enabled, hide originals (meshes will be created by toggle system)
                let show_vertical_air = planet_settings.show_vertical_air;

                for entity in continent_view_query.iter() {
                    commands.entity(entity).insert(if show_vertical_air {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    });
                }

                for entity in ocean_query.iter() {
                    commands.entity(entity).insert(if show_vertical_air {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    });
                }

                for entity in plate_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in temperature_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in precipitation_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in vertical_air_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }
            }

            ViewTabType::Tectonic => {
                // Show: Tectonic plates ONLY
                // Hide: Continent mesh, Ocean, Temperature meshes, Precipitation meshes, Vertical air

                for entity in continent_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in ocean_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in plate_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }

                for entity in temperature_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in precipitation_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in vertical_air_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }

            ViewTabType::Temperature => {
                // Show: Temperature meshes ONLY
                // Hide: Continent mesh, Ocean, Tectonic plates, Precipitation meshes, Vertical air

                for entity in continent_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in ocean_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in plate_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in temperature_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }

                for entity in precipitation_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in vertical_air_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }

            ViewTabType::Precipitations => {
                // Show: Precipitation meshes ONLY
                // Hide: Continent mesh, Ocean, Tectonic plates, Temperature meshes, Vertical air

                for entity in continent_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in ocean_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in plate_view_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in temperature_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in precipitation_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }

                for entity in vertical_air_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
        }
    }
}
