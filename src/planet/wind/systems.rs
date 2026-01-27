// Wind particle systems

use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use super::WindParticleSettings;
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

/// Marker component for wind particle visualization
#[derive(Component)]
pub struct WindParticle {
    pub index: u32,
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
        wind_settings.particle_count = planet_settings.wind_particle_count;
        wind_settings.enabled = planet_settings.show_wind;
    }
}

/// Handle wind tab activation/deactivation
pub fn handle_wind_tab_events(
    mut wind_tab_events: MessageReader<WindTabActiveEvent>,
    mut planet_settings: ResMut<PlanetGenerationSettings>,
    mut existing_particles: Query<Entity, With<WindParticle>>,
    mut commands: Commands,
) {
    for event in wind_tab_events.read() {
        planet_settings.show_wind = event.active;

        // Despawn wind particles when switching away from wind tab
        if !event.active {
            for entity in existing_particles.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Spawn wind particles around the planet
pub fn spawn_wind_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    existing_particles: Query<Entity, With<WindParticle>>,
    settings: Res<WindParticleSettings>,
    planet_settings: Res<PlanetGenerationSettings>,
) {
    if !settings.enabled || !existing_particles.is_empty() {
        return;
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    let particle_count = planet_settings.wind_particle_count;
    info!("Spawning {} wind particles at TRULY random positions", particle_count);

    let sphere_mesh = meshes.add(Sphere::new(0.3).mesh().ico(2).unwrap());
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.8),
        emissive: LinearRgba::rgb(1.0, 1.0, 0.8) * 2.0,
        ..default()
    });

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;
    let lifetime_min = planet_settings.wind_particle_lifetime_min;
    let lifetime_max = planet_settings.wind_particle_lifetime_max;

    let mut rng = rand::rng();

    for i in 0..particle_count as u32 {
        // TRULY RANDOM position using proper RNG
        let position = random_point_on_sphere(&mut rng, sphere_radius as f64);

        // Variable lifetime for each particle
        let lifetime = rng.random_range(lifetime_min..=lifetime_max);

        // Stagger initial ages so they don't all spawn at once
        let initial_age = rng.random_range(0.0..lifetime * 0.95);

        commands.entity(planet_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position),
                WindParticle {
                    index: i,
                    age: initial_age,
                    lifetime,
                },
            ));
        });
    }
}

/// Update particle lifecycle - age, fade in/out, respawn
pub fn update_particle_lifecycle(
    settings: Res<WindParticleSettings>,
    planet_settings: Res<PlanetGenerationSettings>,
    time: Res<Time>,
    mut particles: Query<(
        &mut WindParticle,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.enabled {
        return;
    }

    let delta = time.delta_secs();
    let sphere_radius = settings.planet_radius + settings.particle_height_offset;
    let lifetime_min = planet_settings.wind_particle_lifetime_min;
    let lifetime_max = planet_settings.wind_particle_lifetime_max;

    let fade_in_time = 0.3;
    let fade_out_time = 0.5;

    let mut rng = rand::rng();

    for (mut particle, mut transform, material_handle) in particles.iter_mut() {
        // Update age
        particle.age += delta;

        // Respawn if lifetime exceeded
        if particle.age >= particle.lifetime {
            particle.age = 0.0;

            // New random lifetime
            particle.lifetime = rng.random_range(lifetime_min..=lifetime_max);

            // TRULY RANDOM position using proper RNG - different EVERY respawn!
            transform.translation = random_point_on_sphere(&mut rng, sphere_radius as f64);
        }

        // Calculate alpha based on age (fade in/out)
        let alpha = if particle.age < fade_in_time {
            particle.age / fade_in_time
        } else if particle.age > particle.lifetime - fade_out_time {
            (particle.lifetime - particle.age) / fade_out_time
        } else {
            1.0
        };

        // Update material alpha
        if let Some(material) = materials.get_mut(material_handle.id()) {
            let base_emissive = LinearRgba::rgb(1.0, 1.0, 0.8) * 2.0;
            material.emissive = base_emissive * alpha;
            material.base_color = Color::srgba(1.0, 1.0, 0.8, alpha);
        }
    }
}

/// Generate truly random point on sphere using proper RNG
pub fn random_point_on_sphere<R: Rng + ?Sized>(rng: &mut R, radius: f64) -> Vec3 {
    let u: f64 = rng.random_range(-1.0..=1.0);      // cos(theta)
    let phi: f64 = rng.random_range(0.0..(2.0 * PI as f64));
    let t = (1.0 - u * u).sqrt();

    Vec3 {
        x: (radius * t * phi.cos()) as f32,
        y: (radius * u) as f32,
        z: (radius * t * phi.sin()) as f32,
    }
}
