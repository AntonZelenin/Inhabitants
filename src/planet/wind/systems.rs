// Wind particle systems

use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use super::{WindParticleSettings, PARTICLE_COUNT};
use super::velocity::WindField;
use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

/// Marker component for wind particle visualization
#[derive(Component)]
pub struct WindParticle {
    pub velocity: Vec3,
    pub latitudinal_speed: f32, // Current latitudinal velocity component
    pub age: f32,
    pub lifetime: f32,
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
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.8),
        emissive: LinearRgba::rgb(1.0, 1.0, 0.8) * 2.0,
        ..default()
    });

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    // Spawn particles at random positions on sphere
    for _ in 0..PARTICLE_COUNT {
        let direction = random_sphere_point();
        let position = direction * sphere_radius;

        // Get initial latitudinal speed based on latitude
        let latitudinal_speed = WindField::get_desired_latitudinal_speed(direction);

        // Get initial velocity from wind field
        let velocity = WindField::get_velocity(direction, latitudinal_speed, settings.zonal_speed);

        // Use lifespan from settings with ±20% variation
        let time_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        let mut rng = rand::rngs::StdRng::seed_from_u64(time_seed);
        let variation = rng.random_range(0.8..1.2);
        let lifetime = settings.particle_lifespan * variation;

        // Random initial age for staggered spawning
        let age: f32 = rng.random_range(0.0..lifetime);

        commands.entity(planet_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position),
                WindParticle {
                    velocity,
                    latitudinal_speed,
                    age,
                    lifetime,
                },
            ));
        });
    }
}

/// Generate random point on sphere surface with latitude-based weighting
/// Spawns more particles at source latitudes (30°) and fewer at sink latitudes (equator, poles)
fn random_sphere_point() -> Vec3 {
    // Get system time for random seed
    let time_seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut rng = rand::rngs::StdRng::seed_from_u64(time_seed);

    loop {
        // Generate random point on sphere
        let u: f32 = rng.random();
        let v: f32 = rng.random();

        let theta = u * 2.0 * std::f32::consts::PI;
        let phi = (2.0 * v - 1.0).acos();

        let x = phi.sin() * theta.cos();
        let y = phi.sin() * theta.sin();
        let z = phi.cos();

        let point = Vec3::new(x, y, z).normalize();

        // Get absolute latitude in degrees
        let lat_deg = point.y.asin().to_degrees().abs();

        // Weight function: higher near 30° (source), lower near 0° and 90° (sinks)
        let weight = if lat_deg < 30.0 {
            // Ramp up from equator (0°) to 30°
            // At 0°: weight = 0.2, at 30°: weight = 1.0
            0.2 + 0.8 * (lat_deg / 30.0)
        } else if lat_deg < 60.0 {
            // Ramp down from 30° to 60°
            // At 30°: weight = 1.0, at 60°: weight = 0.3
            1.0 - 0.7 * ((lat_deg - 30.0) / 30.0)
        } else {
            // Stay low near poles (60° to 90°)
            // At 60°: weight = 0.3, at 90°: weight = 0.1
            0.3 - 0.2 * ((lat_deg - 60.0) / 30.0)
        };

        // Accept point with probability proportional to weight
        if rng.random::<f32>() < weight {
            return point;
        }
    }
}


/// Update particle positions and handle respawning
pub fn update_particles(
    mut particles: Query<(&mut Transform, &mut WindParticle)>,
    time: Res<Time>,
    settings: Res<WindParticleSettings>,
) {
    if !settings.enabled {
        return;
    }

    let delta = time.delta_secs();
    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    for (mut transform, mut particle) in particles.iter_mut() {
        // Update age
        particle.age += delta;

        // Check if particle should respawn
        if particle.age >= particle.lifetime {
            // Respawn at new random position
            let direction = random_sphere_point();
            let position = direction * sphere_radius;

            // Reset latitudinal speed to desired at new position
            particle.latitudinal_speed = WindField::get_desired_latitudinal_speed(direction);

            // Get new velocity
            particle.velocity = WindField::get_velocity(direction, particle.latitudinal_speed, settings.zonal_speed);

            // New lifetime based on settings with ±20% variation
            let time_seed = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
            let mut rng = rand::rngs::StdRng::seed_from_u64(time_seed);
            let variation = rng.random_range(0.8..1.2);
            particle.lifetime = settings.particle_lifespan * variation;
            particle.age = 0.0;

            transform.translation = position;
        } else {
            // Get current position direction
            let new_direction = transform.translation.normalize();

            // Calculate desired latitudinal speed at current position
            let desired_speed = WindField::get_desired_latitudinal_speed(new_direction);

            // Relax towards desired speed
            particle.latitudinal_speed = WindField::update_latitudinal_speed(
                particle.latitudinal_speed,
                desired_speed,
                delta
            );

            // Update velocity with new latitudinal component
            particle.velocity = WindField::get_velocity(new_direction, particle.latitudinal_speed, settings.zonal_speed);

            // Move particle along velocity
            let current_pos = transform.translation;
            let new_pos = current_pos + particle.velocity * delta;

            // Project back onto sphere surface
            transform.translation = new_pos.normalize() * sphere_radius;
        }
    }
}
