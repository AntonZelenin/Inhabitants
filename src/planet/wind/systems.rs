// Wind particle systems

use crate::planet::components::PlanetEntity;
use crate::planet::events::{PlanetSpawnedEvent, WindTabActiveEvent};
use crate::planet::resources::{CurrentPlanetData, PlanetGenerationSettings};
use super::{WindParticleSettings, PARTICLE_COUNT};
use bevy::prelude::*;
use rand::Rng;
use planetgen::wind::WindCubeMap as PlanetgenWindCubeMap;

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
    commands.insert_resource(cubemap);
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

        commands.insert_resource(WindCubeMap { inner: wind_map });
        info!("Wind cubemap rebuilt with terrain deflection");
    }
}

