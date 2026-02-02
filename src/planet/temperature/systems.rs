// Temperature visualization systems

use crate::planet::components::{PlanetEntity, TemperatureView};
use crate::planet::events::TemperatureTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use super::TemperatureSettings;
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
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
pub fn initialize_temperature_cubemap(
    mut commands: Commands,
    settings: Res<TemperatureSettings>,
) {
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
    continent_query: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), With<crate::planet::components::ContinentViewMesh>>,
    ocean_query: Query<(Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>), With<crate::planet::components::OceanEntity>>,
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
                        planet_settings.continent_threshold,
                    );
                    let temp_mesh_handle = meshes.add(temp_mesh);

                    // Create solid unlit material for temperature colors
                    let temp_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true, // Show temperature colors without lighting
                        ..default()
                    });

                    let temp_entity = commands.spawn((
                        Mesh3d(temp_mesh_handle),
                        MeshMaterial3d(temp_material),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        TemperatureMesh,
                    )).id();

                    commands.entity(planet_entity).add_child(temp_entity);
                }
            }

            // Hide original ocean mesh and create temperature-colored copy (no edges)
            for (entity, mesh_handle, _material) in ocean_query.iter() {
                commands.entity(entity).insert(Visibility::Hidden);

                if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
                    // Ocean gets temperature colors but no edge detection
                    let temp_mesh = create_simple_temperature_mesh(original_mesh, &temperature_cubemap);
                    let temp_mesh_handle = meshes.add(temp_mesh);

                    // Create solid unlit material for temperature colors
                    let temp_material = materials.add(StandardMaterial {
                        base_color: Color::WHITE,
                        unlit: true, // Show temperature colors without lighting
                        ..default()
                    });

                    let temp_entity = commands.spawn((
                        Mesh3d(temp_mesh_handle),
                        MeshMaterial3d(temp_material),
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::Visible,
                        TemperatureMesh,
                    )).id();

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

/// Spawn temperature visualization mesh
pub fn spawn_temperature_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    existing_meshes: Query<Entity, With<TemperatureMesh>>,
    settings: Res<TemperatureSettings>,
    temperature_cubemap: Res<TemperatureCubeMap>,
) {
    // Only spawn if enabled and not already spawned
    if !settings.enabled || !existing_meshes.is_empty() {
        return;
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    info!("Spawning temperature visualization mesh");

    // Generate a sphere mesh with vertex colors based on temperature
    let mesh = generate_temperature_sphere_mesh(&temperature_cubemap, settings.planet_radius);
    let mesh_handle = meshes.add(mesh);

    // Create material that uses vertex colors
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    let temperature_mesh_entity = commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            TemperatureMesh,
            TemperatureView, // Marker component for visibility toggling
        ))
        .id();

    // Make the temperature mesh a child of the planet entity
    commands.entity(planet_entity).add_child(temperature_mesh_entity);
}

/// Generate a sphere mesh with vertex colors based on temperature data
fn generate_temperature_sphere_mesh(
    temperature_cubemap: &TemperatureCubeMap,
    radius: f32,
) -> Mesh {
    let subdivisions = 64; // Higher resolution for smooth appearance

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    // Generate vertices using spherical coordinates
    for lat_idx in 0..=subdivisions {
        let lat = (lat_idx as f32 / subdivisions as f32) * std::f32::consts::PI;

        for lon_idx in 0..=subdivisions {
            let lon = (lon_idx as f32 / subdivisions as f32) * 2.0 * std::f32::consts::PI;

            // Calculate position on unit sphere
            let x = lat.sin() * lon.cos();
            let y = lat.cos();
            let z = lat.sin() * lon.sin();

            let direction = Vec3::new(x, y, z);
            let position = direction * radius;

            // Sample color from temperature cubemap
            let color = temperature_cubemap.sample_color(direction);

            positions.push([position.x, position.y, position.z]);
            normals.push([direction.x, direction.y, direction.z]);
            colors.push([color.x, color.y, color.z, 1.0]);
        }
    }

    // Generate indices for triangles
    for lat in 0..subdivisions {
        for lon in 0..subdivisions {
            let first = lat * (subdivisions + 1) + lon;
            let second = first + subdivisions + 1;

            // First triangle
            indices.push(first as u32);
            indices.push(second as u32);
            indices.push((first + 1) as u32);

            // Second triangle
            indices.push(second as u32);
            indices.push((second + 1) as u32);
            indices.push((first + 1) as u32);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Create a copy of a mesh with temperature-based vertex colors and continent edge outlines
fn create_temperature_colored_mesh(
    original_mesh: &Mesh,
    temperature_cubemap: &TemperatureCubeMap,
    planet_radius: f32,
    continent_threshold: f32,
) -> Mesh {
    let mut new_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    // Copy positions
    if let Some(positions_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(positions) = positions_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());

            // Coastline radius = planet_radius + continent_threshold
            let coastline_radius = planet_radius + continent_threshold;

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

/// Detect vertices that are near coastlines (transition between land and water elevation)
fn detect_coastline_vertices(
    positions: &[[f32; 3]],
    planet_radius: f32,
    continent_threshold: f32,
) -> std::collections::HashSet<usize> {
    use std::collections::HashSet;

    let mut coastline_vertices = HashSet::new();

    // Coastline radius = planet_radius + continent_threshold
    let continent_radius = planet_radius + continent_threshold;

    // Margin for edge detection (tune this value to adjust edge thickness)
    let margin = planet_radius * 0.01; // 1% of planet radius

    // Debug: track min/max radius
    let mut min_radius = f32::MAX;
    let mut max_radius = f32::MIN;

    for (idx, &[x, y, z]) in positions.iter().enumerate() {
        let vertex_radius = (x * x + y * y + z * z).sqrt();

        min_radius = min_radius.min(vertex_radius);
        max_radius = max_radius.max(vertex_radius);

        // Mark vertices where: continent_radius - margin < vertex_radius < continent_radius + margin
        if vertex_radius > (continent_radius - margin) && vertex_radius < (continent_radius + margin) {
            coastline_vertices.insert(idx);
        }
    }

    info!("Mesh radius range: {:.2} to {:.2}, planet_radius: {:.2}, continent_threshold: {:.2}, coastline_radius: {:.2}±{:.2}",
          min_radius, max_radius, planet_radius, continent_threshold, continent_radius, margin);

    coastline_vertices
}
