use crate::planet::components::{ContinentView, OceanEntity, TectonicPlateView};
use crate::planet::events::{TabSwitchEvent, ViewTabType};
use crate::planet::temperature::systems::TemperatureMesh;
use bevy::prelude::*;

/// CENTRALIZED tab visibility handler - handles ALL tab switching in ONE place
/// Wind particles are managed by their own systems (handle_wind_tab_events + spawn_debug_particles)
pub fn handle_tab_visibility(
    mut tab_switch_events: MessageReader<TabSwitchEvent>,
    continent_view_query: Query<Entity, With<ContinentView>>,
    ocean_query: Query<Entity, With<OceanEntity>>,
    plate_view_query: Query<Entity, With<TectonicPlateView>>,
    temperature_mesh_query: Query<Entity, With<TemperatureMesh>>,
    mut commands: Commands,
) {
    for event in tab_switch_events.read() {
        info!("Switching to {:?} tab - handling ALL visibility", event.tab);

        match event.tab {
            ViewTabType::Continent | ViewTabType::Wind => {
                // Show: Continent mesh + Ocean
                // Hide: Tectonic plates, Temperature meshes
                // Wind particles are managed by handle_wind_tab_events + spawn_debug_particles

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
            }

            ViewTabType::Tectonic => {
                // Show: Tectonic plates ONLY
                // Hide: Continent mesh, Ocean, Temperature meshes

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
            }

            ViewTabType::Temperature => {
                // Show: Temperature meshes ONLY
                // Hide: Continent mesh, Ocean, Tectonic plates

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
            }
        }
    }
}
