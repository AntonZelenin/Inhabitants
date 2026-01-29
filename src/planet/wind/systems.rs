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
    existing_particles: Query<Entity, With<DebugWindParticle>>,
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

    info!("Spawning {} debug wind particles with random positions", PARTICLE_COUNT);

    let sphere_mesh = meshes.add(Sphere::new(0.3).mesh().ico(2).unwrap());
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.8),
        emissive: LinearRgba::rgb(1.0, 1.0, 0.8) * 2.0,
        ..default()
    });

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    // Spawn particles at random positions on sphere
    // Note: The actual positions and lifetimes are managed by the GPU compute shader
    // These debug spheres will be positioned at origin and should ideally be updated
    // from GPU buffer data, but for now we spawn them randomly
    for i in 0..PARTICLE_COUNT {
        let direction = random_sphere_point(i);
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

/// Generate random point on sphere surface using PCG hash
fn random_sphere_point(seed: u32) -> Vec3 {
    let u = pcg_random(seed) as f32 / u32::MAX as f32;
    let v = pcg_random(seed.wrapping_add(1)) as f32 / u32::MAX as f32;

    let theta = u * 2.0 * std::f32::consts::PI;
    let phi = (2.0 * v - 1.0).acos();

    let x = phi.sin() * theta.cos();
    let y = phi.sin() * theta.sin();
    let z = phi.cos();

    Vec3::new(x, y, z).normalize()
}

/// PCG hash function for pseudo-random number generation
fn pcg_random(input: u32) -> u32 {
    let state = input.wrapping_mul(747796405).wrapping_add(2891336453);
    let word = ((state >> ((state >> 28) + 4)) ^ state).wrapping_mul(277803737);
    (word >> 22) ^ word
}

