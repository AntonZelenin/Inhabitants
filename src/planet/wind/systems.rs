use crate::planet::components::PlanetEntity;
use crate::planet::events::{PlanetSpawnedEvent, WindTabActiveEvent};
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::ui::systems::ViewTab;
use crate::planet::wind::components::{WindParticle, WindView};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub fn spawn_wind_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut wind_tab_events: MessageReader<WindTabActiveEvent>,
    mut planet_spawned_events: MessageReader<PlanetSpawnedEvent>,
    view_tab: Option<Res<ViewTab>>,
    settings: Res<PlanetGenerationSettings>,
    existing_particles: Query<Entity, With<WindParticle>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
) {
    let mut should_spawn = false;

    // Check if wind tab was activated
    for event in wind_tab_events.read() {
        if event.active {
            should_spawn = true;
        }
    }

    // Check if planet was spawned while wind tab is active
    for _ in planet_spawned_events.read() {
        if let Some(tab) = view_tab.as_ref() {
            if **tab == ViewTab::Wind {
                should_spawn = true;
            }
        }
    }

    // Also spawn if wind tab is active, particles don't exist, and planet exists
    // This handles respawning after settings change (when update_wind_particles despawns old particles)
    if !should_spawn {
        if let Some(tab) = view_tab.as_ref() {
            if **tab == ViewTab::Wind && existing_particles.is_empty() && !planet_query.is_empty() {
                should_spawn = true;
            }
        }
    }

    if !should_spawn {
        return;
    }

    // Check if planet exists
    let Some(planet_entity) = planet_query.iter().next() else {
        info!("Skip spawning wind particles: no planet entity found");
        return;
    };

    let existing_count = existing_particles.iter().count();
    if existing_count > 0 {
        return; // Already have particles, don't spawn more
    }

    info!("Spawning wind particles on planet surface with 3 latitude zones");

    let planet_radius = settings.radius;
    let height_offset = settings.wind_particle_height_offset;
    let particle_sphere_radius = planet_radius + height_offset;

    // Spawn 3 separate particle effects for the 3 wind zones
    // This allows each zone to have different movement patterns
    spawn_wind_zone_particles(
        &mut commands,
        &mut effects,
        planet_entity,
        &settings,
        particle_sphere_radius,
        "tropical",
        0.0,
        30.0,
    );

    spawn_wind_zone_particles(
        &mut commands,
        &mut effects,
        planet_entity,
        &settings,
        particle_sphere_radius,
        "temperate",
        30.0,
        60.0,
    );

    spawn_wind_zone_particles(
        &mut commands,
        &mut effects,
        planet_entity,
        &settings,
        particle_sphere_radius,
        "polar",
        60.0,
        90.0,
    );

    info!("Wind particles spawned for all 3 latitude zones");
}

/// Spawn particles for a specific latitude zone
/// min_lat/max_lat: latitude range in degrees (0 = equator, 90 = pole)
/// Wind direction is calculated automatically based on atmospheric circulation cells
fn spawn_wind_zone_particles(
    commands: &mut Commands,
    effects: &mut Assets<EffectAsset>,
    planet_entity: Entity,
    settings: &PlanetGenerationSettings,
    particle_sphere_radius: f32,
    zone_name: &str,
    min_lat: f32,
    max_lat: f32,
) {
    let planet_radius = settings.radius;

    // Calculate target latitude based on atmospheric circulation cells
    let mid_lat = (min_lat + max_lat) / 2.0;
    let target_latitude = if mid_lat < 30.0 {
        // Hadley cell: move toward equator
        0.0
    } else if mid_lat < 60.0 {
        // Ferrel cell: move toward poles
        90.0
    } else {
        // Polar cell: move back toward 60°
        60.0
    };

    // Particle count for this zone (divide total by 3 zones)
    let zone_particle_count = settings.wind_particle_count / 3;

    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1.0, 1.0, 0.8, 0.0));
    color_gradient.add_key(0.1, Vec4::new(1.0, 1.0, 0.8, 1.0));
    color_gradient.add_key(0.9, Vec4::new(1.0, 1.0, 0.8, 1.0));
    color_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 0.8, 0.0));

    let mut size_gradient = bevy_hanabi::Gradient::new();
    let base_size = settings.wind_particle_mesh_size * planet_radius * 0.02;
    size_gradient.add_key(0.0, Vec3::splat(base_size));
    size_gradient.add_key(1.0, Vec3::splat(base_size * 0.8));

    let avg_lifetime = (settings.wind_particle_lifetime_min + settings.wind_particle_lifetime_max) / 2.0;

    let writer = ExprWriter::new();

    let init_age = SetAttributeModifier::new(
        Attribute::AGE,
        writer.rand(ScalarType::Float).mul(writer.lit(avg_lifetime)).expr(),
    );

    let lifetime = writer.lit(avg_lifetime).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Spawn particles on sphere surface constrained to latitude band
    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(particle_sphere_radius).expr(),
        dimension: ShapeDimension::Surface,
    };

    // Calculate velocity direction based on zone
    // TODO: Add north-south (meridional) component toward target_latitude
    // Currently: simple tangential velocity for east-west movement
    // Future: Add complex velocity calculation considering:
    //   - Meridional flow (toward target_latitude calculated above)
    //   - Coriolis deflection (from planet rotation)
    //   - Terrain channeling (mountains deflect, valleys funnel)

    let init_vel = SetVelocityTangentModifier {
        origin: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed: writer.lit(settings.wind_speed).expr(),
    };

    // Use a rate spawner that will spawn all particles in the first frame
    // Rate = particles_per_second, so set it very high to spawn all immediately
    // Hanabi will naturally cap at the effect capacity and then maintain the count
    let spawn_rate = zone_particle_count as f32 * 100.0; // Spawn 100x count per second = all in 0.01s
    let spawner = SpawnerSettings::rate(CpuValue::Single(spawn_rate));

    // Keep particles constrained to sphere surface as they move
    // This pulls particles back to the sphere if they drift away from the surface
    let conform = ConformToSphereModifier {
        origin: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(particle_sphere_radius).expr(),
        influence_dist: writer.lit(particle_sphere_radius * 0.1).expr(),
        attraction_accel: writer.lit(5.0).expr(),
        max_attraction_speed: writer.lit(10.0).expr(),
        shell_half_thickness: Some(writer.lit(0.5).expr()),
        sticky_factor: Some(writer.lit(0.5).expr()),
    };

    let effect = EffectAsset::new(
        zone_particle_count as u32 + 512,
        spawner,
        writer.finish(),
    )
    .with_name(format!("wind_particles_{}", zone_name))
    .with_simulation_space(SimulationSpace::Local)
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .update(conform)
    .render(ColorOverLifetimeModifier::new(color_gradient))
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient,
        screen_space_size: false,
    })
    .render(OrientModifier::new(OrientMode::FaceCameraPosition));

    let effect_handle = effects.add(effect);

    // Spawn particles as child of planet so they rotate with it
    commands.entity(planet_entity).with_children(|parent| {
        parent.spawn((
            Name::new(format!("wind_particles_{}", zone_name)),
            ParticleEffect::new(effect_handle),
            Transform::IDENTITY,
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            WindParticle {
                position: Vec3::ZERO,
                velocity: Vec3::ZERO,
                age: 0.0,
                lifetime: 1.0,
                particle_id: 0,
                respawn_count: 0,
                trail_positions: std::collections::VecDeque::new(),
            },
            WindView,
        ));
    });

    info!("Spawned {} particles for {} zone ({}° - {}°, target: {}°)", zone_particle_count, zone_name, min_lat, max_lat, target_latitude);
}

/// Update wind particles - respawn when settings change
/// This ensures particles reflect updated wind speed and other settings
pub fn update_wind_particles(
    mut commands: Commands,
    settings: Res<PlanetGenerationSettings>,
    particle_query: Query<(Entity, &ParticleEffect), With<WindParticle>>,
) {
    // If settings changed while wind is visible, despawn and let spawn system recreate
    if settings.is_changed() && settings.show_wind {
        let count = particle_query.iter().count();
        if count > 0 {
            info!("Wind settings changed, despawning {} particle effects for respawn", count);
            for (entity, _) in particle_query.iter() {
                commands.entity(entity).despawn();
            }
            // Respawn will happen automatically next frame via spawn_wind_particles
            // which checks if particles exist before spawning
        }
    }
}

/// Despawn wind particles when switching away from wind tab
pub fn despawn_wind_particles(
    mut commands: Commands,
    mut events: MessageReader<WindTabActiveEvent>,
    particles: Query<Entity, With<WindParticle>>,
) {
    for event in events.read() {
        if !event.active {
            let count = particles.iter().count();
            if count > 0 {
                info!("Despawning {} wind particle effects", count);
                for entity in particles.iter() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

// Future wind simulation system (commented out - reserved for terrain interaction)
//
// CPU-side wind simulation system
// Currently uses multiple Hanabi effects for different latitude zones.
//
// Future enhancements will require:
// 1. Custom compute shader for particle updates (GPU-side calculation)
// 2. Terrain-aware particle deflection (sample height map in shader)
// 3. Coriolis effect from planet rotation
// 4. Atmospheric pressure and temperature gradients
//
// Architecture notes:
// - Hanabi particles are GPU-driven, can't easily modify from CPU per-particle
// - Options for complex behavior:
//   A) Multiple effects per zone (current approach - simple but limited)
//   B) Custom compute shader (best for complex physics, requires shader coding)
//   C) CPU-spawned mesh particles (flexible but lower performance for 1000s of particles)
//   D) Hybrid: Hanabi for rendering, CPU for logic with periodic respawn
/*
pub fn simulate_wind_particles(
    time: Res<Time>,
    settings: Res<PlanetGenerationSettings>,
    particles: Query<Entity, With<WindParticle>>,
) {
    // This system is reserved for future CPU-side simulation
    // Current implementation uses Hanabi's built-in modifiers for movement

    // When we need terrain interaction or complex physics:
    // 1. Add custom compute shader to Hanabi effect
    // 2. Pass terrain data as texture/buffer to shader
    // 3. Calculate forces in shader based on particle position vs terrain
    // 4. Apply Coriolis, pressure gradients, turbulence in same shader
}

/// Helper function to calculate wind velocity based on particle position
/// Returns velocity vector in local planet space
fn calculate_wind_velocity_at_position(
    position: Vec3,
    _planet_radius: f32,
    wind_speed: f32,
) -> Vec3 {
    // Normalize position to get direction from planet center
    let dir = position.normalize();

    // Calculate latitude (0 = equator, ±90 = poles)
    // Y component of normalized direction gives sin(latitude)
    let sin_lat = dir.y;
    let latitude_deg = sin_lat.asin().to_degrees();
    let abs_lat = latitude_deg.abs();

    // Determine wind zone and direction
    // 0-30°: toward equator (Hadley cell - trade winds)
    // 30-60°: toward poles (Ferrel cell - westerlies)
    // 60-90°: toward 60° (Polar cell - polar easterlies)

    let target_lat_deg = if abs_lat < 30.0 {
        // Trade winds: move toward equator (0°)
        0.0
    } else if abs_lat < 60.0 {
        // Westerlies: move toward poles (90°)
        90.0 * latitude_deg.signum()
    } else {
        // Polar easterlies: move toward 60°
        60.0 * latitude_deg.signum()
    };

    // Calculate tangent vector (perpendicular to radial direction, in Y plane)
    // This creates north-south movement
    let up = Vec3::Y;
    let east = dir.cross(up).normalize();
    let north = east.cross(dir).normalize();

    // Calculate direction to target latitude
    let current_lat_rad = latitude_deg.to_radians();
    let target_lat_rad = target_lat_deg.to_radians();
    let lat_diff = target_lat_rad - current_lat_rad;

    // Add east-west component (planet rotation, Coriolis effect - for future)
    // For now, just meridional (north-south) flow
    let velocity = north * lat_diff.signum() * wind_speed;

    // Future enhancements:
    // 1. Add Coriolis deflection: velocity += east * coriolis_factor
    // 2. Add terrain influence: adjust velocity based on nearby mountains/valleys
    // 3. Add pressure gradients: modify speed based on atmospheric pressure
    // 4. Add turbulence: add noise for realistic variation

    velocity
}
*/
