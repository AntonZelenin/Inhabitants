use crate::planet::components::{ContinentView, OceanEntity, TectonicPlateView};
use crate::planet::events::{TabSwitchEvent, ViewTabType};
use crate::planet::temperature::systems::TemperatureMesh;
use crate::planet::wind::systems::WindParticle;
use bevy::prelude::*;

/// CENTRALIZED tab visibility handler - handles ALL tab switching in ONE place
/// This is the ONLY system that should control visibility of tab-specific entities
pub fn handle_tab_visibility(
    mut tab_switch_events: MessageReader<TabSwitchEvent>,
    continent_view_query: Query<Entity, With<ContinentView>>,
    ocean_query: Query<Entity, With<OceanEntity>>,
    plate_view_query: Query<Entity, With<TectonicPlateView>>,
    temperature_mesh_query: Query<Entity, With<TemperatureMesh>>,
    wind_particle_query: Query<Entity, With<WindParticle>>,
    mut commands: Commands,
) {
    for event in tab_switch_events.read() {
        info!("Switching to {:?} tab - handling ALL visibility", event.tab);

        match event.tab {
            ViewTabType::Continent => {
                // Show: Continent mesh + Ocean
                // Hide: Tectonic plates, Temperature meshes, Wind particles

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

                for entity in wind_particle_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }

            ViewTabType::Wind => {
                // Show: Continent mesh + Ocean + Wind particles
                // Hide: Tectonic plates, Temperature meshes

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

                for entity in wind_particle_query.iter() {
                    commands.entity(entity).insert(Visibility::Visible);
                }
            }

            ViewTabType::Tectonic => {
                // Show: Tectonic plates ONLY
                // Hide: Continent mesh, Ocean, Temperature meshes, Wind particles

                let continent_count = continent_view_query.iter().count();
                let ocean_count = ocean_query.iter().count();
                let plate_count = plate_view_query.iter().count();

                info!("TECTONIC VIEW: Found {} continent entities, {} ocean entities, {} plate entities",
                    continent_count, ocean_count, plate_count);

                for entity in continent_view_query.iter() {
                    info!("Hiding continent entity: {:?}", entity);
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                // EXPLICITLY HIDE OCEAN - NO FUCKING OCEAN IN TECTONIC VIEW!!!
                for entity in ocean_query.iter() {
                    info!("HIDING OCEAN ENTITY: {:?}", entity);
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in plate_view_query.iter() {
                    info!("Showing plate entity: {:?}", entity);
                    commands.entity(entity).insert(Visibility::Visible);
                }

                for entity in temperature_mesh_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }

                for entity in wind_particle_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }

            ViewTabType::Temperature => {
                // Show: Temperature meshes ONLY
                // Hide: Continent mesh, Ocean, Tectonic plates, Wind particles

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

                for entity in wind_particle_query.iter() {
                    commands.entity(entity).insert(Visibility::Hidden);
                }
            }
        }
    }
}
