use super::{PreviousPrecipitationSettings, PrecipitationSettings};
use crate::planet::components::PlanetEntity;
use crate::planet::events::PrecipitationTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::temperature::systems::TemperatureCubeMap;
use crate::planet::wind::systems::VerticalAirCubeMap;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use planetgen::precipitations::{PrecipitationCubeMap as PlanetgenPrecipitationCubeMap, precipitation_to_color};

/// Bevy-compatible PrecipitationCubeMap resource
#[derive(Resource, Clone)]
pub struct PrecipitationCubeMap {
    inner: PlanetgenPrecipitationCubeMap,
}

impl PrecipitationCubeMap {
    pub fn build(
        vertical_air: &planetgen::wind::VerticalAirCubeMap,
        temperature: Option<&planetgen::temperature::TemperatureCubeMap>,
        temperature_weight: f32,
        equator_temp: f32,
        pole_temp: f32,
    ) -> Self {
        let inner = PlanetgenPrecipitationCubeMap::build(
            vertical_air,
            temperature,
            temperature_weight,
            equator_temp,
            pole_temp,
        );
        Self { inner }
    }

    pub fn sample(&self, position: Vec3) -> f32 {
        self.inner.sample(position)
    }

    pub fn sample_color(&self, position: Vec3) -> Vec3 {
        let value = self.inner.sample(position);
        precipitation_to_color(value)
    }
}

/// Marker component for precipitation visualization mesh
#[derive(Component)]
pub struct PrecipitationMesh;

/// Initialize the precipitation cube map resource at startup
pub fn initialize_precipitation_cubemap(
    mut commands: Commands,
    settings: Res<PrecipitationSettings>,
    planet_settings: Res<PlanetGenerationSettings>,
    vertical_air: Option<Res<VerticalAirCubeMap>>,
    temperature: Option<Res<TemperatureCubeMap>>,
) {
    info!("Initializing precipitation cube map...");

    // We need the vertical air map to build precipitation
    // If it doesn't exist yet, create a placeholder that will be rebuilt later
    if let Some(vertical_air) = vertical_air {
        let temp_inner = temperature.as_ref().map(|t| &t.inner);
        let cubemap = PrecipitationCubeMap::build(
            &vertical_air.inner,
            temp_inner,
            settings.temperature_weight,
            planet_settings.temperature_equator_temp,
            planet_settings.temperature_pole_temp,
        );
        commands.insert_resource(cubemap);
    } else {
        info!("Vertical air map not available yet, precipitation map will be built later");
    }
}

/// Update precipitation settings from planet generation settings
pub fn update_precipitation_settings(
    planet_settings: Res<PlanetGenerationSettings>,
    mut previous_settings: ResMut<PreviousPrecipitationSettings>,
    mut precipitation_settings: ResMut<PrecipitationSettings>,
    mut precipitation_cubemap: Option<ResMut<PrecipitationCubeMap>>,
    vertical_air: Option<Res<VerticalAirCubeMap>>,
    temperature: Option<Res<TemperatureCubeMap>>,
    mut commands: Commands,
) {
    // Always update basic settings
    precipitation_settings.planet_radius = planet_settings.radius;
    precipitation_settings.enabled = planet_settings.show_precipitation;
    precipitation_settings.temperature_weight = planet_settings.precipitation_temperature_weight;

    // Check if precipitation-related values have changed
    let precip_changed =
        previous_settings.0.precipitation_temperature_weight != planet_settings.precipitation_temperature_weight ||
        previous_settings.0.precipitation_cubemap_resolution != planet_settings.precipitation_cubemap_resolution;

    // Rebuild cubemap if settings changed or if vertical air map was updated
    let vertical_air_changed = vertical_air.as_ref().map_or(false, |v| v.is_changed());
    let temperature_changed = temperature.as_ref().map_or(false, |t| t.is_changed());

    if precip_changed || vertical_air_changed || temperature_changed {
        if let Some(vertical_air) = vertical_air {
            info!("Rebuilding precipitation cubemap with new settings...");
            let temp_inner = temperature.as_ref().map(|t| &t.inner);
            let new_cubemap = PrecipitationCubeMap::build(
                &vertical_air.inner,
                temp_inner,
                planet_settings.precipitation_temperature_weight,
                planet_settings.temperature_equator_temp,
                planet_settings.temperature_pole_temp,
            );

            if let Some(ref mut cubemap) = precipitation_cubemap {
                **cubemap = new_cubemap;
            } else {
                commands.insert_resource(new_cubemap);
            }

            // Update tracking
            previous_settings.0.precipitation_temperature_weight = planet_settings.precipitation_temperature_weight;
            previous_settings.0.precipitation_cubemap_resolution = planet_settings.precipitation_cubemap_resolution;
        }
    }
}

/// Regenerate precipitation meshes when cubemap changes
pub fn regenerate_precipitation_meshes_on_settings_change(
    planet_settings: Res<PlanetGenerationSettings>,
    previous_settings: Res<PreviousPrecipitationSettings>,
    precipitation_cubemap: Option<Res<PrecipitationCubeMap>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    continent_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::ContinentViewMesh>,
    >,
    ocean_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::OceanEntity>,
    >,
    existing_precip_meshes: Query<Entity, With<PrecipitationMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // Only regenerate if precipitation view is active
    if !planet_settings.show_precipitation {
        return;
    }

    let Some(ref precipitation_cubemap) = precipitation_cubemap else {
        return;
    };

    // Regenerate meshes if cubemap changed
    if !precipitation_cubemap.is_changed() && !previous_settings.is_changed() {
        return;
    }

    info!("Regenerating precipitation meshes due to settings change");

    // Despawn existing precipitation meshes
    for entity in existing_precip_meshes.iter() {
        commands.entity(entity).despawn();
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    // Recreate continent precipitation mesh
    for (_entity, mesh_handle, _material) in continent_query.iter() {
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let precip_mesh = create_precipitation_colored_mesh(original_mesh, precipitation_cubemap);
            let precip_mesh_handle = meshes.add(precip_mesh);

            let precip_material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            let precip_entity = commands
                .spawn((
                    Mesh3d(precip_mesh_handle),
                    MeshMaterial3d(precip_material),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    PrecipitationMesh,
                    crate::planet::components::PrecipitationView,
                ))
                .id();

            commands.entity(planet_entity).add_child(precip_entity);
        }
    }

    // Recreate ocean precipitation mesh
    for (_entity, mesh_handle, _material) in ocean_query.iter() {
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let precip_mesh = create_precipitation_colored_mesh(original_mesh, precipitation_cubemap);
            let precip_mesh_handle = meshes.add(precip_mesh);

            let precip_material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            let precip_entity = commands
                .spawn((
                    Mesh3d(precip_mesh_handle),
                    MeshMaterial3d(precip_material),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    PrecipitationMesh,
                    crate::planet::components::PrecipitationView,
                ))
                .id();

            commands.entity(planet_entity).add_child(precip_entity);
        }
    }
}

/// Handle precipitation tab activation/deactivation
pub fn handle_precipitation_tab_events(
    mut precipitation_tab_events: MessageReader<PrecipitationTabActiveEvent>,
    mut planet_settings: ResMut<PlanetGenerationSettings>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    continent_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::ContinentViewMesh>,
    >,
    ocean_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::OceanEntity>,
    >,
    existing_precip_meshes: Query<Entity, With<PrecipitationMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    precipitation_cubemap: Option<Res<PrecipitationCubeMap>>,
    mut commands: Commands,
) {
    for event in precipitation_tab_events.read() {
        planet_settings.show_precipitation = event.active;

        if event.active {
            // Only create precipitation meshes if they don't already exist
            if !existing_precip_meshes.is_empty() {
                info!("Precipitation meshes already exist, skipping creation");
                continue;
            }

            let Some(ref precipitation_cubemap) = precipitation_cubemap else {
                warn!("Precipitation cubemap not available");
                continue;
            };

            info!("Creating precipitation-colored mesh copies");

            let Some(planet_entity) = planet_query.iter().next() else {
                warn!("No planet entity found");
                continue;
            };

            // Create precipitation mesh for continent
            for (_entity, mesh_handle, _material) in continent_query.iter() {
                if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
                    let precip_mesh = create_precipitation_colored_mesh(original_mesh, precipitation_cubemap);
                    let precip_mesh_handle = meshes.add(precip_mesh);

                    let precip_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true,
                        ..default()
                    });

                    let precip_entity = commands
                        .spawn((
                            Mesh3d(precip_mesh_handle),
                            MeshMaterial3d(precip_material),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::Visible,
                            PrecipitationMesh,
                            crate::planet::components::PrecipitationView,
                        ))
                        .id();

                    commands.entity(planet_entity).add_child(precip_entity);
                }
            }

            // Create precipitation mesh for ocean
            for (_entity, mesh_handle, _material) in ocean_query.iter() {
                if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
                    let precip_mesh = create_precipitation_colored_mesh(original_mesh, precipitation_cubemap);
                    let precip_mesh_handle = meshes.add(precip_mesh);

                    let precip_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true,
                        ..default()
                    });

                    let precip_entity = commands
                        .spawn((
                            Mesh3d(precip_mesh_handle),
                            MeshMaterial3d(precip_material),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::Visible,
                            PrecipitationMesh,
                            crate::planet::components::PrecipitationView,
                        ))
                        .id();

                    commands.entity(planet_entity).add_child(precip_entity);
                }
            }
        } else {
            info!("Hiding precipitation-colored mesh copies");

            // Hide precipitation mesh copies
            for entity in existing_precip_meshes.iter() {
                commands.entity(entity).try_insert(Visibility::Hidden);
            }
        }
    }
}

/// Create a copy of a mesh with precipitation-based vertex colors
fn create_precipitation_colored_mesh(
    original_mesh: &Mesh,
    precipitation_cubemap: &PrecipitationCubeMap,
) -> Mesh {
    let mut new_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Copy positions and generate colors
    if let Some(positions_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(positions) = positions_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());

            let colors: Vec<[f32; 4]> = positions
                .iter()
                .map(|&[x, y, z]| {
                    let position = Vec3::new(x, y, z);
                    let color = precipitation_cubemap.sample_color(position);
                    [color.x, color.y, color.z, 1.0]
                })
                .collect();

            new_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
    }

    // Copy normals
    if let Some(normals_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        if let Some(normals) = normals_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.to_vec());
        }
    }

    // Copy indices
    if let Some(indices) = original_mesh.indices() {
        new_mesh.insert_indices(indices.clone());
    }

    new_mesh
}
