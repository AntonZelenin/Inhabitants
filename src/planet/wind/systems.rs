// Wind particle systems

use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use super::{WindParticleSettings, PARTICLE_COUNT};
use super::velocity::WindField;
use bevy::prelude::*;
use rand::Rng;

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
        wind_settings.density_bin_deg = planet_settings.wind_density_bin_deg;
        wind_settings.density_pressure_strength = planet_settings.wind_density_pressure_strength;
        wind_settings.uplift_zone_deg = planet_settings.wind_uplift_zone_deg;
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

    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    let mut rng = rand::rng();

    // Spawn particles at random positions on sphere
    for _ in 0..PARTICLE_COUNT {
        let direction = random_sphere_point(&mut rng);
        let position = direction * sphere_radius;

        // Get initial latitudinal speed based on latitude
        let latitudinal_speed = WindField::get_desired_latitudinal_speed(direction);

        // Get initial velocity from wind field
        let velocity = WindField::get_velocity(direction, latitudinal_speed, settings.zonal_speed);

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
                    latitudinal_speed,
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

fn latitude_degrees(direction: Vec3) -> f32 {
    direction.y.asin().to_degrees()
}

fn lat_to_bin(lat_deg: f32, bin_size_deg: f32, bin_count: usize) -> usize {
    let clamped = lat_deg.clamp(-90.0, 90.0);
    let raw = ((clamped + 90.0) / bin_size_deg).floor() as isize;
    raw.clamp(0, (bin_count.saturating_sub(1)) as isize) as usize
}

fn respawn_particle(
    particle: &mut WindParticle,
    transform: &mut Transform,
    settings: &WindParticleSettings,
    sphere_radius: f32,
    rng: &mut impl Rng,
) {
    let direction = random_sphere_point(rng);
    let position = direction * sphere_radius;

    particle.latitudinal_speed = WindField::get_desired_latitudinal_speed(direction);
    particle.velocity = WindField::get_velocity(direction, particle.latitudinal_speed, settings.zonal_speed);

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
) {
    if !settings.enabled {
        return;
    }

    let delta = time.delta_secs();
    let sphere_radius = settings.planet_radius + settings.particle_height_offset;

    let bin_size_deg = settings.density_bin_deg.max(0.1);
    let mut bin_count = (180.0 / bin_size_deg).ceil() as usize;
    if bin_count == 0 {
        bin_count = 1;
    }

    let mut density = vec![0u32; bin_count];
    for transform in particles.p0().iter() {
        let direction = transform.translation.normalize();
        let lat_deg = latitude_degrees(direction);
        let bin = lat_to_bin(lat_deg, bin_size_deg, bin_count);
        density[bin] = density[bin].saturating_add(1);
    }

    let mut rng = rand::rng();

    for (mut transform, mut particle) in particles.p1().iter_mut() {
        particle.age += delta;

        let direction = transform.translation.normalize();
        let lat_deg = latitude_degrees(direction);

        if particle.age >= particle.lifetime {
            respawn_particle(&mut particle, &mut transform, &settings, sphere_radius, &mut rng);
            continue;
        }

        let desired_speed = WindField::get_desired_latitudinal_speed(direction);

        // Apply density-based pressure force
        let bin = lat_to_bin(lat_deg, bin_size_deg, bin_count);
        let north_bin = (bin + 1).min(bin_count - 1);
        let south_bin = bin.saturating_sub(1);
        let north_density = density[north_bin] as f32;
        let south_density = density[south_bin] as f32;
        let current_density = density[bin].max(1) as f32;

        let pressure_gradient = (south_density - north_density) / current_density;
        let pressure_speed = pressure_gradient * settings.density_pressure_strength;

        // Apply uplift zone force (reduces latitudinal speed to simulate vertical lift)
        let in_uplift_zone = lat_deg.abs() <= settings.uplift_zone_deg;
        let uplift_damping = if in_uplift_zone {
            // Smoothly reduce latitudinal movement in uplift zone
            let zone_center_dist = lat_deg.abs() / settings.uplift_zone_deg.max(0.1);
            let damping_factor = 1.0 - (1.0 - zone_center_dist).powi(2); // Quadratic falloff
            damping_factor.clamp(0.3, 1.0) // Keep some movement
        } else {
            1.0
        };

        let desired_with_pressure = (desired_speed + pressure_speed) * uplift_damping;

        particle.latitudinal_speed = WindField::update_latitudinal_speed(
            particle.latitudinal_speed,
            desired_with_pressure,
            delta,
        );

        particle.velocity = WindField::get_velocity(direction, particle.latitudinal_speed, settings.zonal_speed);

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

