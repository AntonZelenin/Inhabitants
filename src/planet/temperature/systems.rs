use super::{PreviousPlanetSettings, TemperatureSettings};
use crate::planet::components::PlanetEntity;
use crate::planet::events::TemperatureTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{PrimitiveTopology};
use bevy::prelude::*;
use planetgen::temperature::TemperatureCubeMap as PlanetgenTemperatureCubeMap;

/// Bevy-compatible TemperatureCubeMap resource
#[derive(Resource, Clone)]
pub struct TemperatureCubeMap {
    pub inner: PlanetgenTemperatureCubeMap,
}

impl TemperatureCubeMap {
    pub fn build(resolution: usize, equator_temp: f32, pole_temp: f32, min_temp: f32, max_temp: f32, falloff: f32) -> Self {
        let inner = PlanetgenTemperatureCubeMap::build_with_falloff(resolution, equator_temp, pole_temp, min_temp, max_temp, falloff);
        Self { inner }
    }

    pub fn sample_temperature(&self, position: Vec3) -> f32 {
        self.inner.sample_temperature(position)
    }

    pub fn sample_color(&self, position: Vec3) -> Vec3 {
        self.inner.sample_color(position)
    }
}

/// Marker component for temperature visualization mesh
#[derive(Component)]
pub struct TemperatureMesh;

/// Initialize the temperature cube map resource at startup
pub fn initialize_temperature_cubemap(mut commands: Commands, settings: Res<TemperatureSettings>) {
    info!("Initializing temperature cube map...");
    let config = planetgen::get_config();
    let cubemap = TemperatureCubeMap::build(
        settings.temperature_cubemap_resolution,
        config.temperature.equator_temp,
        config.temperature.pole_temp,
        config.temperature.min_temp,
        config.temperature.max_temp,
        config.temperature.latitude_falloff,
    );
    commands.insert_resource(cubemap);
}

/// Update temperature settings from planet generation settings
/// Only rebuilds the cubemap when temperature values actually change
pub fn update_temperature_settings(
    planet_settings: Res<PlanetGenerationSettings>,
    mut previous_settings: ResMut<PreviousPlanetSettings>,
    mut temperature_settings: ResMut<TemperatureSettings>,
    mut temperature_cubemap: ResMut<TemperatureCubeMap>,
) {
    // Always update these basic settings
    temperature_settings.planet_radius = planet_settings.radius;
    temperature_settings.enabled = planet_settings.show_temperature;

    // Check if temperature-related values have actually changed
    let temp_changed =
        previous_settings.0.temperature_equator_temp != planet_settings.temperature_equator_temp ||
        previous_settings.0.temperature_pole_temp != planet_settings.temperature_pole_temp ||
        previous_settings.0.temperature_max_temp != planet_settings.temperature_max_temp ||
        previous_settings.0.temperature_min_temp != planet_settings.temperature_min_temp ||
        previous_settings.0.temperature_latitude_falloff != planet_settings.temperature_latitude_falloff ||
        previous_settings.0.temperature_cubemap_resolution != planet_settings.temperature_cubemap_resolution;

    // Only rebuild cubemap if temperature values actually changed
    if temp_changed {
        info!("Rebuilding temperature cubemap with new settings...");
        *temperature_cubemap = TemperatureCubeMap::build(
            planet_settings.temperature_cubemap_resolution,
            planet_settings.temperature_equator_temp,
            planet_settings.temperature_pole_temp,
            planet_settings.temperature_min_temp,
            planet_settings.temperature_max_temp,
            planet_settings.temperature_latitude_falloff,
        );

        // Update the previous settings to track current values
        previous_settings.0.temperature_equator_temp = planet_settings.temperature_equator_temp;
        previous_settings.0.temperature_pole_temp = planet_settings.temperature_pole_temp;
        previous_settings.0.temperature_max_temp = planet_settings.temperature_max_temp;
        previous_settings.0.temperature_min_temp = planet_settings.temperature_min_temp;
        previous_settings.0.temperature_latitude_falloff = planet_settings.temperature_latitude_falloff;
        previous_settings.0.temperature_cubemap_resolution = planet_settings.temperature_cubemap_resolution;
    }

    // Check if land_temperature_bonus changed (doesn't require cubemap rebuild, just mesh update)
    if previous_settings.0.land_temperature_bonus != planet_settings.land_temperature_bonus {
        previous_settings.0.land_temperature_bonus = planet_settings.land_temperature_bonus;
    }
}

/// Regenerate temperature meshes when cubemap OR land_temperature_bonus changes
pub fn regenerate_temperature_meshes_on_settings_change(
    planet_settings: Res<PlanetGenerationSettings>,
    previous_settings: Res<PreviousPlanetSettings>,
    temperature_cubemap: Res<TemperatureCubeMap>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    continent_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::ContinentViewMesh>,
    >,
    ocean_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::OceanEntity>,
    >,
    existing_temp_meshes: Query<Entity, With<TemperatureMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    // Only regenerate if temperature view is active
    if !planet_settings.show_temperature {
        return;
    }

    // Regenerate meshes if cubemap OR land_temperature_bonus changed
    // Both trigger the same action: regenerate the temperature-colored meshes
    if !temperature_cubemap.is_changed() && !previous_settings.is_changed() {
        return;
    }

    info!("Regenerating temperature meshes due to settings change");

    // Despawn existing temperature meshes
    for entity in existing_temp_meshes.iter() {
        commands.entity(entity).despawn();
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    // Recreate continent temperature mesh
    for (_entity, mesh_handle, _material) in continent_query.iter() {
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let temp_mesh = create_temperature_colored_mesh(
                original_mesh,
                &temperature_cubemap,
                planet_settings.radius,
                planet_settings.continent_threshold,
                planet_settings.land_temperature_bonus,
                planet_settings.temperature_min_temp,
                planet_settings.temperature_max_temp,
            );
            let temp_mesh_handle = meshes.add(temp_mesh);

            let temp_material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            let temp_entity = commands
                .spawn((
                    Mesh3d(temp_mesh_handle),
                    MeshMaterial3d(temp_material),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    TemperatureMesh,
                    crate::planet::components::TemperatureView,
                ))
                .id();

            commands.entity(planet_entity).add_child(temp_entity);
        }
    }

    // Recreate ocean temperature mesh
    for (_entity, mesh_handle, _material) in ocean_query.iter() {
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let temp_mesh = create_simple_temperature_mesh(original_mesh, &temperature_cubemap);
            let temp_mesh_handle = meshes.add(temp_mesh);

            let temp_material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            let temp_entity = commands
                .spawn((
                    Mesh3d(temp_mesh_handle),
                    MeshMaterial3d(temp_material),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    TemperatureMesh,
                    crate::planet::components::TemperatureView,
                ))
                .id();

            commands.entity(planet_entity).add_child(temp_entity);
        }
    }
}

/// Handle temperature tab activation/deactivation
pub fn handle_temperature_tab_events(
    mut temperature_tab_events: MessageReader<TemperatureTabActiveEvent>,
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
    existing_temp_meshes: Query<Entity, With<TemperatureMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    temperature_cubemap: Res<TemperatureCubeMap>,
    mut commands: Commands,
) {
    for event in temperature_tab_events.read() {
        planet_settings.show_temperature = event.active;

        if event.active {
            // Only create temperature meshes if they don't already exist
            if !existing_temp_meshes.is_empty() {
                info!("Temperature meshes already exist, skipping creation");
                continue;
            }

            info!("Creating temperature-colored mesh copies");

            let Some(planet_entity) = planet_query.iter().next() else {
                warn!("No planet entity found");
                continue;
            };

            // Hide original continent mesh and create temperature-colored copy
            for (_entity, mesh_handle, _material) in continent_query.iter() {
                // DO NOT manipulate visibility - centralized system handles it
                // Just create the temperature mesh copy

                if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
                    let temp_mesh = create_temperature_colored_mesh(
                        original_mesh,
                        &temperature_cubemap,
                        planet_settings.radius,
                        planet_settings.continent_threshold,
                        planet_settings.land_temperature_bonus,
                        planet_settings.temperature_min_temp,
                        planet_settings.temperature_max_temp,
                    );
                    let temp_mesh_handle = meshes.add(temp_mesh);

                    // Create solid unlit material for temperature colors
                    let temp_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true, // Show temperature colors without lighting
                        ..default()
                    });

                    let temp_entity = commands
                        .spawn((
                            Mesh3d(temp_mesh_handle),
                            MeshMaterial3d(temp_material),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::Visible,
                            TemperatureMesh,
                            crate::planet::components::TemperatureView, // Add marker for visibility control
                        ))
                        .id();

                    commands.entity(planet_entity).add_child(temp_entity);
                }
            }

            // Hide original ocean mesh and create temperature-colored copy (no edges)
            for (_entity, mesh_handle, _material) in ocean_query.iter() {
                // DO NOT manipulate visibility - centralized system handles it
                // Just create the temperature mesh copy

                if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
                    // Ocean gets temperature colors but no edge detection
                    let temp_mesh =
                        create_simple_temperature_mesh(original_mesh, &temperature_cubemap);
                    let temp_mesh_handle = meshes.add(temp_mesh);

                    // Create solid unlit material for temperature colors
                    let temp_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true, // Show temperature colors without lighting
                        ..default()
                    });

                    let temp_entity = commands
                        .spawn((
                            Mesh3d(temp_mesh_handle),
                            MeshMaterial3d(temp_material),
                            Transform::default(),
                            GlobalTransform::default(),
                            Visibility::Visible,
                            TemperatureMesh,
                            crate::planet::components::TemperatureView, // Add marker for visibility control
                        ))
                        .id();

                    commands.entity(planet_entity).add_child(temp_entity);
                }
            }
        } else {
            info!("Hiding temperature-colored mesh copies");

            // Hide temperature mesh copies instead of despawning
            for entity in existing_temp_meshes.iter() {
                commands.entity(entity).try_insert(Visibility::Hidden);
            }

            // DO NOT manipulate continent or ocean visibility here!
            // The centralized tab visibility system handles ALL mesh visibility
        }
    }
}

/// Create a copy of a mesh with temperature-based vertex colors and continent darkening
fn create_temperature_colored_mesh(
    original_mesh: &Mesh,
    temperature_cubemap: &TemperatureCubeMap,
    planet_radius: f32,
    continent_threshold: f32,
    land_temperature_bonus: f32,
    min_temp: f32,
    max_temp: f32,
) -> Mesh {
    let mut new_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Copy positions
    if let Some(positions_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(positions) = positions_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());

            // Ocean level is now at planet_radius + continent_threshold
            let ocean_level = planet_radius + continent_threshold;

            // Generate temperature colors, applying land bonus and darkening land
            let colors: Vec<[f32; 4]> = positions
                .iter()
                .map(|&[x, y, z]| {
                    let position = Vec3::new(x, y, z);
                    let direction = position.normalize();
                    let vertex_radius = (x * x + y * y + z * z).sqrt();

                    // Check if this is land (above ocean level)
                    let is_land = vertex_radius > ocean_level;

                    // Get base temperature from latitude
                    let base_temp = temperature_cubemap.sample_temperature(direction);

                    // Apply land temperature bonus if on land
                    let adjusted_temp = if is_land {
                        base_temp + land_temperature_bonus
                    } else {
                        base_temp
                    };

                    // Get color for the adjusted temperature
                    let mut color = planetgen::temperature::TemperatureField::temperature_to_color(
                        adjusted_temp,
                        min_temp,
                        max_temp,
                    );

                    // Darken land vertices for visual distinction
                    if is_land {
                        color *= 0.3; // Darken to 30%
                    }

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

/// Create a simple temperature-colored mesh without edge detection (for ocean)
fn create_simple_temperature_mesh(
    original_mesh: &Mesh,
    temperature_cubemap: &TemperatureCubeMap,
) -> Mesh {
    let mut new_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Copy positions
    if let Some(positions_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(positions) = positions_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());

            // Generate temperature colors without edges
            let colors: Vec<[f32; 4]> = positions
                .iter()
                .map(|&[x, y, z]| {
                    let position = Vec3::new(x, y, z);
                    let direction = position.normalize();
                    let color = temperature_cubemap.sample_color(direction);
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
