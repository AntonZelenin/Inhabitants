use super::TemperatureSettings;
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
    inner: PlanetgenTemperatureCubeMap,
}

impl TemperatureCubeMap {
    pub fn build(resolution: usize) -> Self {
        let inner = PlanetgenTemperatureCubeMap::build(resolution);
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
    let cubemap = TemperatureCubeMap::build(settings.temperature_cubemap_resolution);
    commands.insert_resource(cubemap);
}

/// Update temperature settings from planet generation settings
pub fn update_temperature_settings(
    planet_settings: Res<PlanetGenerationSettings>,
    mut temperature_settings: ResMut<TemperatureSettings>,
) {
    if planet_settings.is_changed() {
        temperature_settings.planet_radius = planet_settings.radius;
        temperature_settings.enabled = planet_settings.show_temperature;
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
            info!("Creating temperature-colored mesh copies");

            let Some(planet_entity) = planet_query.iter().next() else {
                warn!("No planet entity found");
                continue;
            };

            // Hide original continent mesh and create temperature-colored copy
            for (entity, mesh_handle, _material) in continent_query.iter() {
                commands.entity(entity).insert(Visibility::Hidden);

                if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
                    let temp_mesh = create_temperature_colored_mesh(
                        original_mesh,
                        &temperature_cubemap,
                        planet_settings.radius,
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
                        ))
                        .id();

                    commands.entity(planet_entity).add_child(temp_entity);
                }
            }

            // Hide original ocean mesh and create temperature-colored copy (no edges)
            for (entity, mesh_handle, _material) in ocean_query.iter() {
                commands.entity(entity).insert(Visibility::Hidden);

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
                        ))
                        .id();

                    commands.entity(planet_entity).add_child(temp_entity);
                }
            }
        } else {
            info!("Removing temperature-colored mesh copies");

            // Despawn temperature mesh copies
            for entity in existing_temp_meshes.iter() {
                commands.entity(entity).despawn();
            }

            // Show original meshes again
            for (entity, _, _) in continent_query.iter() {
                commands.entity(entity).insert(Visibility::Visible);
            }
            for (entity, _, _) in ocean_query.iter() {
                commands.entity(entity).insert(Visibility::Visible);
            }
        }
    }
}

/// Create a copy of a mesh with temperature-based vertex colors and continent edge outlines
fn create_temperature_colored_mesh(
    original_mesh: &Mesh,
    temperature_cubemap: &TemperatureCubeMap,
    planet_radius: f32,
) -> Mesh {
    let mut new_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Copy positions
    if let Some(positions_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(positions) = positions_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());

            let coastline_radius = planet_radius;

            // Generate temperature colors, darkening everything above coastline
            let colors: Vec<[f32; 4]> = positions
                .iter()
                .map(|&[x, y, z]| {
                    let position = Vec3::new(x, y, z);
                    let direction = position.normalize();
                    let vertex_radius = (x * x + y * y + z * z).sqrt();

                    let mut color = temperature_cubemap.sample_color(direction);

                    // Darken vertices above coastline
                    if vertex_radius > coastline_radius {
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
