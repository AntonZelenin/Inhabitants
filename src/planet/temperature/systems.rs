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
    existing_meshes: Query<Entity, With<TemperatureMesh>>,
    mut commands: Commands,
) {
    for event in temperature_tab_events.read() {
        planet_settings.show_temperature = event.active;

        // Despawn temperature mesh when switching away from temperature tab
        if !event.active {
            for entity in existing_meshes.iter() {
                commands.entity(entity).despawn();
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
