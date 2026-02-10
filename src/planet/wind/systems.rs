// Wind particle systems

use crate::planet::components::{PlanetEntity, VerticalAirView};
use crate::planet::events::{PlanetSpawnedEvent, WindTabActiveEvent};
use crate::planet::resources::{CurrentPlanetData, PlanetGenerationSettings};
use super::{WindParticleSettings, PARTICLE_COUNT};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use rand::Rng;
use planetgen::wind::WindCubeMap as PlanetgenWindCubeMap;
use planetgen::wind::VerticalAirCubeMap as PlanetgenVerticalAirCubeMap;
use planetgen::wind::vertical::divergence_to_color;

/// Bevy-compatible WindCubeMap resource
#[derive(Resource, Clone)]
pub struct WindCubeMap {
    inner: PlanetgenWindCubeMap,
}

impl WindCubeMap {
    pub fn build(resolution: usize, zonal_speed: f32) -> Self {
        let inner = PlanetgenWindCubeMap::build(resolution, zonal_speed);
        Self { inner }
    }

    pub fn sample(&self, position: Vec3) -> Vec3 {
        self.inner.sample(position)
    }
}

/// Bevy-compatible VerticalAirCubeMap resource
#[derive(Resource, Clone)]
pub struct VerticalAirCubeMap {
    inner: PlanetgenVerticalAirCubeMap,
}

impl VerticalAirCubeMap {
    pub fn build_from_wind(wind_inner: &PlanetgenWindCubeMap) -> Self {
        let inner = PlanetgenVerticalAirCubeMap::build_from_wind(wind_inner);
        Self { inner }
    }

    pub fn sample(&self, position: Vec3) -> f32 {
        self.inner.sample(position)
    }
}

/// Marker component for vertical air movement overlay mesh
#[derive(Component)]
pub struct VerticalAirMesh;

/// Marker component for wind particle visualization
#[derive(Component)]
pub struct WindParticle {
    pub velocity: Vec3,
    pub latitudinal_speed: f32, // Current latitudinal velocity component
    pub age: f32,
    pub lifetime: f32,
}

/// Initialize the wind cube map resource at startup
pub fn initialize_wind_cubemap(
    mut commands: Commands,
    settings: Res<WindParticleSettings>,
) {
    info!("Initializing wind cube map...");
    let cubemap = WindCubeMap::build(settings.wind_cubemap_resolution, settings.zonal_speed);
    let vertical = VerticalAirCubeMap::build_from_wind(&cubemap.inner);
    commands.insert_resource(cubemap);
    commands.insert_resource(vertical);
}

/// Update wind particle settings from planet generation settings
pub fn update_wind_settings(
    planet_settings: Res<PlanetGenerationSettings>,
    mut wind_settings: ResMut<WindParticleSettings>,
) {
    if planet_settings.is_changed() {
        wind_settings.planet_radius = planet_settings.radius;
        wind_settings.particle_height_offset = planet_settings.wind_particle_height_offset;
        wind_settings.enabled = planet_settings.show_wind;
        wind_settings.zonal_speed = planet_settings.wind_zonal_speed;
        wind_settings.particle_lifespan = planet_settings.wind_particle_lifespan;
        wind_settings.show_vertical_air = planet_settings.show_vertical_air;
    }
}

/// Handle wind tab activation/deactivation
pub fn handle_wind_tab_events(
    mut wind_tab_events: MessageReader<WindTabActiveEvent>,
    mut planet_settings: ResMut<PlanetGenerationSettings>,
    existing_particles: Query<Entity, With<WindParticle>>,
    mut commands: Commands,
) {
    for event in wind_tab_events.read() {
        planet_settings.show_wind = event.active;

        // Despawn debug particles when switching away from wind tab
        if !event.active {
            for entity in existing_particles.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawn wind particle visualization spheres
pub fn spawn_debug_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    existing_particles: Query<Entity, With<WindParticle>>,
    settings: Res<WindParticleSettings>,
    wind_cubemap: Res<WindCubeMap>,
) {
    // Only spawn if enabled and not already spawned
    if !settings.enabled || !existing_particles.is_empty() {
        return;
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    info!("Spawning {} wind particles with random positions", PARTICLE_COUNT);

    let sphere_mesh = meshes.add(Sphere::new(0.3).mesh().ico(2).unwrap());

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    let mut rng = rand::rng();

    // Spawn particles at random positions on sphere
    for _ in 0..PARTICLE_COUNT {
        let direction = random_sphere_point(&mut rng);
        let position = direction * sphere_radius;

        // Get initial velocity from pre-computed wind cube map
        let velocity = wind_cubemap.sample(direction);

        // Use lifespan from settings with ±20% variation
        let variation = rng.random_range(0.8..1.2);
        let lifetime = settings.particle_lifespan * variation;

        // Random initial age for staggered spawning
        let age: f32 = rng.random_range(0.0..lifetime);

        // Create material with alpha blending enabled
        let material = materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 0.8, 1.0),
            emissive: LinearRgba::rgb(1.0, 1.0, 0.8) * 2.0,
            alpha_mode: AlphaMode::Blend,
            ..default()
        });

        commands.entity(planet_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(position),
                WindParticle {
                    velocity,
                    latitudinal_speed: 0.0, // No longer used, kept for compatibility
                    age,
                    lifetime,
                },
            ));
        });
    }
}

/// Generate random point on sphere surface (uniform distribution)
fn random_sphere_point(rng: &mut impl Rng) -> Vec3 {
    let u: f32 = rng.random();
    let v: f32 = rng.random();

    let theta = u * 2.0 * std::f32::consts::PI;
    let phi = (2.0 * v - 1.0).acos();

    let x = phi.sin() * theta.cos();
    let y = phi.sin() * theta.sin();
    let z = phi.cos();

    Vec3::new(x, y, z).normalize()
}

fn respawn_particle(
    particle: &mut WindParticle,
    transform: &mut Transform,
    settings: &WindParticleSettings,
    sphere_radius: f32,
    wind_cubemap: &WindCubeMap,
    rng: &mut impl Rng,
) {
    let direction = random_sphere_point(rng);
    let position = direction * sphere_radius;

    // Get wind velocity from pre-computed cube map
    let velocity = wind_cubemap.sample(direction);

    particle.latitudinal_speed = 0.0; // No longer needed, kept for compatibility
    particle.velocity = velocity;

    let variation = rng.random_range(0.8..1.2);
    particle.lifetime = settings.particle_lifespan * variation;
    particle.age = 0.0;

    transform.translation = position;
}

/// Update particle positions and handle respawning
pub fn update_particles(
    mut particles: ParamSet<(
        Query<&Transform, With<WindParticle>>,
        Query<(&mut Transform, &mut WindParticle)>,
    )>,
    time: Res<Time>,
    settings: Res<WindParticleSettings>,
    wind_cubemap: Res<WindCubeMap>,
) {
    if !settings.enabled {
        return;
    }

    let delta = time.delta_secs();
    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    let mut rng = rand::rng();

    for (mut transform, mut particle) in particles.p1().iter_mut() {
        particle.age += delta;

        let direction = transform.translation.normalize();

        if particle.age >= particle.lifetime {
            respawn_particle(&mut particle, &mut transform, &settings, sphere_radius, &wind_cubemap, &mut rng);
            continue;
        }

        // Sample wind velocity from pre-computed cube map
        particle.velocity = wind_cubemap.sample(direction);

        let current_pos = transform.translation;
        let new_pos = current_pos + particle.velocity * delta;

        transform.translation = new_pos.normalize() * sphere_radius;
    }
}

/// Update particle transparency for fade in/out effects
pub fn update_particle_fade(
    mut particles: Query<(&WindParticle, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<WindParticleSettings>,
) {
    if !settings.enabled {
        return;
    }

    for (particle, material_handle) in particles.iter_mut() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let fade_in_progress = if settings.fade_in_duration > 0.0 {
                (particle.age / settings.fade_in_duration).clamp(0.0, 1.0)
            } else {
                1.0
            };

            let time_until_death = particle.lifetime - particle.age;
            let fade_out_progress = if settings.fade_out_duration > 0.0 {
                (time_until_death / settings.fade_out_duration).clamp(0.0, 1.0)
            } else {
                1.0
            };

            // Combine both fade factors (use the minimum to handle both simultaneously)
            let alpha = fade_in_progress.min(fade_out_progress);

            // Update base color alpha
            let mut color = material.base_color.to_srgba();
            color.alpha = alpha;
            material.base_color = color.into();

            // Also fade emissive for consistency
            let emissive_strength = alpha * 2.0; // Original emissive was * 2.0
            material.emissive = LinearRgba::rgb(1.0, 1.0, 0.8) * emissive_strength;
        }
    }
}

/// Rebuild wind cubemap with terrain deflection after a planet is generated.
pub fn rebuild_wind_cubemap_after_planet(
    mut commands: Commands,
    mut events: MessageReader<PlanetSpawnedEvent>,
    planet_data: Res<CurrentPlanetData>,
    settings: Res<WindParticleSettings>,
    planet_settings: Res<PlanetGenerationSettings>,
) {
    for _ in events.read() {
        let Some(ref planet) = planet_data.planet_data else {
            continue;
        };

        let deflection_config = planetgen::config::WindDeflectionConfig {
            height_threshold: planet_settings.wind_deflection_height_threshold,
            height_scale: planet_settings.wind_deflection_height_scale,
            spread_radius: planet_settings.wind_deflection_spread_radius,
            spread_decay: planet_settings.wind_deflection_spread_decay,
            deflection_strength: planet_settings.wind_deflection_strength,
            deflection_iterations: planet_settings.wind_deflection_iterations,
        };
        let (wind_map, _influence) = PlanetgenWindCubeMap::build_with_terrain(
            settings.wind_cubemap_resolution,
            settings.zonal_speed,
            planet,
            &deflection_config,
        );

        let vertical = VerticalAirCubeMap::build_from_wind(&wind_map);
        commands.insert_resource(WindCubeMap { inner: wind_map });
        commands.insert_resource(vertical);
        info!("Wind cubemap rebuilt with terrain deflection");
    }
}

/// Toggle vertical air movement overlay on/off.
/// Creates colored mesh copies when enabled (hiding originals), despawns them when disabled.
pub fn handle_vertical_air_toggle(
    settings: Res<WindParticleSettings>,
    planet_settings: Res<PlanetGenerationSettings>,
    vertical_cubemap: Res<VerticalAirCubeMap>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    continent_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::ContinentViewMesh>,
    >,
    ocean_query: Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::OceanEntity>,
    >,
    continent_view_query: Query<Entity, With<crate::planet::components::ContinentView>>,
    ocean_view_query: Query<Entity, With<crate::planet::components::OceanEntity>>,
    existing_meshes: Query<Entity, With<VerticalAirMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    if !planet_settings.is_changed() && !vertical_cubemap.is_changed() {
        return;
    }

    let should_show = settings.show_vertical_air && settings.enabled;
    let has_meshes = !existing_meshes.is_empty();

    if should_show && !has_meshes {
        spawn_vertical_air_meshes(
            &planet_query, &continent_query, &ocean_query,
            &vertical_cubemap, &mut meshes, &mut materials, &mut commands,
        );
        // Hide original continent + ocean meshes so overlay is visible
        for entity in continent_view_query.iter() {
            commands.entity(entity).insert(Visibility::Hidden);
        }
        for entity in ocean_view_query.iter() {
            commands.entity(entity).insert(Visibility::Hidden);
        }
    } else if !should_show && has_meshes {
        // Despawn overlay and restore original meshes
        for entity in existing_meshes.iter() {
            commands.entity(entity).despawn();
        }
        for entity in continent_view_query.iter() {
            commands.entity(entity).insert(Visibility::Visible);
        }
        for entity in ocean_view_query.iter() {
            commands.entity(entity).insert(Visibility::Visible);
        }
    } else if should_show && has_meshes && vertical_cubemap.is_changed() {
        // Rebuild after wind cubemap changed
        for entity in existing_meshes.iter() {
            commands.entity(entity).despawn();
        }
        spawn_vertical_air_meshes(
            &planet_query, &continent_query, &ocean_query,
            &vertical_cubemap, &mut meshes, &mut materials, &mut commands,
        );
    }
}

/// Helper to spawn vertical air overlay meshes from continent + ocean originals.
fn spawn_vertical_air_meshes(
    planet_query: &Query<Entity, With<PlanetEntity>>,
    continent_query: &Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::ContinentViewMesh>,
    >,
    ocean_query: &Query<
        (Entity, &Mesh3d, &MeshMaterial3d<StandardMaterial>),
        With<crate::planet::components::OceanEntity>,
    >,
    vertical_cubemap: &VerticalAirCubeMap,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    commands: &mut Commands,
) {
    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    info!("Creating vertical air movement overlay");

    for (_entity, mesh_handle, _material) in continent_query.iter() {
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let colored_mesh = create_vertical_air_mesh(original_mesh, vertical_cubemap);
            let mesh_handle = meshes.add(colored_mesh);
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    VerticalAirMesh,
                    VerticalAirView,
                ))
                .id();
            commands.entity(planet_entity).add_child(entity);
        }
    }

    for (_entity, mesh_handle, _material) in ocean_query.iter() {
        if let Some(original_mesh) = meshes.get(&mesh_handle.0) {
            let colored_mesh = create_vertical_air_mesh(original_mesh, vertical_cubemap);
            let mesh_handle = meshes.add(colored_mesh);
            let material = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..default()
            });

            let entity = commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    VerticalAirMesh,
                    VerticalAirView,
                ))
                .id();
            commands.entity(planet_entity).add_child(entity);
        }
    }
}

/// Create a mesh copy with vertex colors based on vertical air movement
fn create_vertical_air_mesh(
    original_mesh: &Mesh,
    vertical_cubemap: &VerticalAirCubeMap,
) -> Mesh {
    let mut new_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    if let Some(positions_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        if let Some(positions) = positions_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.to_vec());

            let colors: Vec<[f32; 4]> = positions
                .iter()
                .map(|&[x, y, z]| {
                    let direction = Vec3::new(x, y, z).normalize();
                    let value = vertical_cubemap.sample(direction);
                    let color = divergence_to_color(value);
                    [color.x, color.y, color.z, 1.0]
                })
                .collect();

            new_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
    }

    if let Some(normals_attr) = original_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        if let Some(normals) = normals_attr.as_float3() {
            new_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.to_vec());
        }
    }

    if let Some(indices) = original_mesh.indices() {
        new_mesh.insert_indices(indices.clone());
    }

    new_mesh
}

