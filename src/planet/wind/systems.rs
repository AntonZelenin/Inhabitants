// Wind particle systems

use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use super::{WindParticleSettings, WindMaterial, WindParticleMaterial, WindTimeUniforms};
use bevy::prelude::*;
use rand::Rng;
use std::f32::consts::PI;

// CONSTANT: All particles have the same lifetime
const PARTICLE_LIFETIME: f32 = 5.0;
const FADE_IN_TIME: f32 = 0.5;
const FADE_OUT_TIME: f32 = 0.5;

/// Marker component for wind particle visualization
#[derive(Component)]
pub struct WindParticle {
    pub index: u32,
    pub spawn_time: f32,  // When this particle was born (in elapsed seconds)
}

/// Update wind particle settings from planet generation settings
pub fn update_wind_settings(
    planet_settings: Res<PlanetGenerationSettings>,
    mut wind_settings: ResMut<WindParticleSettings>,
) {
    wind_settings.enabled = planet_settings.show_wind;
    if planet_settings.is_changed() {
        wind_settings.planet_radius = planet_settings.radius;
        wind_settings.particle_height_offset = planet_settings.wind_particle_height_offset;
        wind_settings.particle_count = planet_settings.wind_particle_count;
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
    mut materials: ResMut<Assets<WindMaterial>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    existing_particles: Query<Entity, With<WindParticle>>,
    settings: Res<WindParticleSettings>,
    planet_settings: Res<PlanetGenerationSettings>,
    time: Res<Time>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    if !settings.enabled || !existing_particles.is_empty() {
        return;
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    let camera_pos = camera_query.single().ok().map(|t| t.translation());

    let particle_count = planet_settings.wind_particle_count;
    info!("Spawning {} wind particles", particle_count);

    // Create shared material with time uniforms
    let material = materials.add(WindMaterial {
        base: StandardMaterial {
            base_color: Color::srgb(0.9, 0.9, 0.9),
            emissive: LinearRgba::rgb(0.95, 0.95, 0.95) * 1.5,
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            cull_mode: None,
            ..default()
        },
        extension: WindParticleMaterial {
            time_uniforms: WindTimeUniforms {
                time_now: time.elapsed_secs(),
                lifetime: PARTICLE_LIFETIME,
                fade_in: FADE_IN_TIME,
                fade_out: FADE_OUT_TIME,
            },
        },
    });

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    let mut rng = rand::rng();
    let current_time = time.elapsed_secs();

    for i in 0..particle_count as u32 {
        // TRULY RANDOM position using proper RNG
        let position = random_point_on_sphere(&mut rng, sphere_radius as f64);

        // Spawn time = current_time - rand(0, PARTICLE_LIFETIME)
        // This makes each particle start at a different point in its lifecycle
        let time_offset = rng.random_range(0.0..PARTICLE_LIFETIME);
        let spawn_time = current_time - time_offset;

        // Create a unique mesh for each particle with spawn_time encoded in vertex color
        let mut circle_mesh = Mesh::from(Circle::new(planet_settings.wind_particle_mesh_size.max(0.01)));

        // Encode spawn_time in vertex color red channel
        let vertex_count = circle_mesh.count_vertices();
        let spawn_time_colors = vec![[spawn_time, 0.0, 0.0, 1.0]; vertex_count];
        circle_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, spawn_time_colors);

        let mesh_handle = meshes.add(circle_mesh);
        let mut transform = Transform::from_translation(position);
        if let Some(camera_pos) = camera_pos {
            transform.look_at(camera_pos, Vec3::Y);
        }

        commands.entity(planet_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material.clone()),
                transform,
                WindParticle {
                    index: i,
                    spawn_time,
                },
            ));
        });
    }
}

/// Update particles with movement AND time uniforms (mimics GPU compute shader behavior)
pub fn update_particle_with_movement(
    settings: Res<WindParticleSettings>,
    time: Res<Time>,
    mut particles: Query<(&mut WindParticle, &mut Transform, &Mesh3d, &MeshMaterial3d<WindMaterial>), With<WindParticle>>,
    mut materials: ResMut<Assets<WindMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    if !settings.enabled {
        return;
    }

    let camera_pos = camera_query.single().ok().map(|t| t.translation());

    let current_time = time.elapsed_secs();
    let delta_time = time.delta_secs();
    let sphere_radius = settings.planet_radius + settings.particle_height_offset;
    let mut rng = rand::rng();

    // Update time uniforms in the shared material (only need to do once per frame)
    let mut material_updated = false;

    for (mut particle, mut transform, mesh_handle, material_handle) in particles.iter_mut() {
        // Update time uniforms once
        if !material_updated {
            if let Some(material) = materials.get_mut(material_handle.id()) {
                material.extension.time_uniforms.time_now = current_time;
                material_updated = true;
            }
        }

        // Calculate age: current_time - spawn_time
        let age = current_time - particle.spawn_time;

        // Respawn if lifetime exceeded
        if age >= PARTICLE_LIFETIME {
            // Respawn NOW - set spawn_time to current moment
            particle.spawn_time = current_time;

            // TRULY RANDOM position using proper RNG - different EVERY respawn!
            transform.translation = random_point_on_sphere(&mut rng, sphere_radius as f64);

            // Update vertex colors to encode new spawn_time
            if let Some(mesh) = meshes.get_mut(mesh_handle.id()) {
                let vertex_count = mesh.count_vertices();
                let spawn_time_colors = vec![[current_time, 0.0, 0.0, 1.0]; vertex_count];
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, spawn_time_colors);
            }
        }

        // === LATITUDE-BASED MOVEMENT (matching GPU compute shader) ===
        let normalized_pos = transform.translation.normalize();

        // Calculate latitude (angle from equator): asin(y)
        let latitude_rad = normalized_pos.y.asin();
        let latitude_deg = latitude_rad.to_degrees().abs();

        // Determine flow direction based on latitude bands
        // 0-30°: toward equator (-1.0)
        // 30-60°: away from equator (+1.0)
        // 60-90°: toward equator (-1.0)
        let flow_direction = if latitude_deg < 30.0 {
            -1.0
        } else if latitude_deg < 60.0 {
            1.0
        } else {
            -1.0
        };

        // Calculate tangent velocity (perpendicular to radial, moving in latitude direction)
        // Create east-west tangent vector
        let up = Vec3::Y;
        let east = up.cross(normalized_pos).normalize_or_zero();

        // Handle poles where cross product is zero
        let east = if east.length_squared() < 0.001 {
            Vec3::X
        } else {
            east
        };

        let north = normalized_pos.cross(east).normalize();

        // Move toward/away from equator based on latitude band
        let speed = 3.0; // meters per second (matches GPU shader)
        let velocity = north * flow_direction * speed;

        // Apply velocity (move particle)
        transform.translation += velocity * delta_time;

        // Keep particle on sphere surface (re-normalize to sphere radius)
        transform.translation = transform.translation.normalize() * sphere_radius;

        // Face the camera (billboard) so circles render as 2D sprites.
        if let Some(camera_pos) = camera_pos {
            transform.look_at(camera_pos, Vec3::Y);
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
