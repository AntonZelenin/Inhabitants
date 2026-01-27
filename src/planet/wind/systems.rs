// Wind particle systems

use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use super::{WindParticleSettings, PARTICLE_COUNT};
use bevy::prelude::*;

/// Marker component for debug particle visualization
#[derive(Component)]
pub struct DebugWindParticle {
    pub index: u32,
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
    }
}

/// Handle wind tab activation/deactivation
pub fn handle_wind_tab_events(
    mut wind_tab_events: MessageReader<WindTabActiveEvent>,
    mut planet_settings: ResMut<PlanetGenerationSettings>,
    mut existing_particles: Query<Entity, With<DebugWindParticle>>,
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

/// Spawn debug visualization spheres for particles
/// This is a temporary solution - particles should be rendered from GPU buffers
pub fn spawn_debug_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
    existing_particles: Query<Entity, With<DebugWindParticle>>,
    settings: Res<WindParticleSettings>,
) {
    // Only spawn if enabled and not already spawned
    if !settings.enabled || !existing_particles.is_empty() {
        return;
    }

    let Some(planet_entity) = planet_query.iter().next() else {
        return;
    };

    info!("Spawning {} debug wind particles", PARTICLE_COUNT);

    let sphere_mesh = meshes.add(Sphere::new(0.3).mesh().ico(2).unwrap());
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.8),
        emissive: LinearRgba::rgb(1.0, 1.0, 0.8) * 2.0,
        ..default()
    });

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    // Spawn particles using Fibonacci sphere distribution
    for i in 0..PARTICLE_COUNT {
        let direction = fibonacci_sphere(i, PARTICLE_COUNT);
        let position = direction * sphere_radius;

        commands.entity(planet_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(sphere_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(position),
                DebugWindParticle { index: i },
            ));
        });
    }
}

// ...existing fibonacci_sphere function...

/// Fibonacci sphere distribution for uniform points on a sphere
fn fibonacci_sphere(i: u32, n: u32) -> Vec3 {
    let phi = std::f32::consts::PI * (5.0_f32.sqrt() - 1.0); // Golden angle
    let y = 1.0 - (i as f32 / (n - 1) as f32) * 2.0; // Y from 1 to -1
    let radius = (1.0 - y * y).sqrt();
    let theta = phi * i as f32;

    let x = theta.cos() * radius;
    let z = theta.sin() * radius;

    Vec3::new(x, y, z).normalize()
}

