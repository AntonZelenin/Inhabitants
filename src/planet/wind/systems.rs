use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use crate::planet::wind::components::{WindParticle, WindView};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

/// DEBUG: Spawn particles directly in front of camera to verify they work
pub fn spawn_wind_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    mut events: MessageReader<WindTabActiveEvent>,
    settings: Res<PlanetGenerationSettings>,
    existing_particles: Query<Entity, With<WindParticle>>,
) {
    // Only process events
    for event in events.read() {
        if !event.active {
            // Tab switched away from wind - handled by despawn system
            continue;
        }

        // Wind tab activated - spawn particles
        let existing_count = existing_particles.iter().count();
        if existing_count > 0 {
            info!("Wind particles already exist ({}), skipping spawn", existing_count);
            continue;
        }

        info!("DEBUG: Wind tab activated, spawning particles on planet surface");

        // Create color gradient - BRIGHT RED for visibility
        let mut color_gradient = bevy_hanabi::Gradient::new();
        color_gradient.add_key(0.0, Vec4::new(1.0, 0.0, 0.0, 1.0)); // Bright red, opaque
        color_gradient.add_key(1.0, Vec4::new(1.0, 0.0, 0.0, 1.0)); // Stay red

        // Create size gradient - LARGE particles
        let mut size_gradient = bevy_hanabi::Gradient::new();
        let base_size = 5.0; // Very large for visibility
        size_gradient.add_key(0.0, Vec3::splat(base_size));
        size_gradient.add_key(1.0, Vec3::splat(base_size));

        let avg_lifetime = 10.0; // Long lifetime for testing

        let writer = ExprWriter::new();

        let init_age = SetAttributeModifier::new(
            Attribute::AGE,
            writer.lit(0.0).expr(),
        );

        let lifetime = writer.lit(avg_lifetime).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

        // Position particles on sphere ABOVE planet surface
        // Planet is at origin with radius from settings
        let planet_radius = settings.radius;
        let height_offset = settings.wind_particle_height_offset;
        let particle_sphere_radius = planet_radius + height_offset + 5.0; // Extra 5 units above to be clearly visible

        info!("DEBUG: Planet radius: {}, Spawning particles at radius: {}", planet_radius, particle_sphere_radius);

        let init_pos = SetPositionSphereModifier {
            center: writer.lit(Vec3::ZERO).expr(), // Planet center is at origin
            radius: writer.lit(particle_sphere_radius).expr(),
            dimension: ShapeDimension::Surface,
        };

        let init_vel = SetVelocitySphereModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            speed: writer.lit(0.0).expr(), // Stationary for debugging
        };

        info!(
            "DEBUG: Spawning 500 particles/sec on sphere surface at radius {} with size {}",
            particle_sphere_radius, base_size
        );

        // Use once spawner to create all particles at startup
        let spawner = SpawnerSettings::once(CpuValue::Single(100.0)); // Just 100 particles for testing

        let effect = EffectAsset::new(
            32768,
            spawner,
            writer.finish(),
        )
        .with_name("debug_wind_particles")
        .with_simulation_space(SimulationSpace::Global) // Global space for debugging
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .render(ColorOverLifetimeModifier::new(color_gradient))
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        })
        .render(OrientModifier::new(OrientMode::FaceCameraPosition));

        let effect_handle = effects.add(effect);

        // Spawn particle effect at origin (planet center)
        // Particles will spawn on sphere surface around it
        commands.spawn((
            Name::new("debug_wind_particles"),
            ParticleEffect::new(effect_handle),
            Transform::IDENTITY, // At origin where planet is
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

        info!("DEBUG: Wind particle effect spawned on sphere at radius {}!", particle_sphere_radius);
    }
}

/* ORIGINAL PRODUCTION CODE - COMMENTED OUT FOR DEBUGGING
pub fn spawn_wind_particles(
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    settings: Res<PlanetGenerationSettings>,
    existing_particles: Query<Entity, With<WindParticle>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
) {
    // Only spawn if wind is enabled and we don't have particles yet
    if !settings.show_wind {
        return;
    }

    let existing_count = existing_particles.iter().count();
    if existing_count > 0 {
        info!("Wind particles already exist ({}), skipping spawn", existing_count);
        return;
    }

    let Ok(planet_entity) = planet_query.single() else {
        warn!("Cannot spawn wind particles: no planet entity found");
        return;
    };

    let radius = settings.radius;
    let height_offset = settings.wind_particle_height_offset;
    let particle_radius = radius + height_offset;

    // Create color gradient - make particles fully opaque and bright
    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(1.0, 1.0, 0.8, 0.0)); // Fade in
    color_gradient.add_key(0.1, Vec4::new(1.0, 1.0, 0.8, 1.0)); // Fully opaque quickly
    color_gradient.add_key(0.9, Vec4::new(1.0, 1.0, 0.8, 1.0)); // Stay opaque
    color_gradient.add_key(1.0, Vec4::new(1.0, 1.0, 0.8, 0.0)); // Fade out

    // Create size gradient - make particles much larger and visible
    let mut size_gradient = bevy_hanabi::Gradient::new();
    // Scale particle size relative to planet radius for visibility
    let base_size = settings.wind_particle_mesh_size * radius * 0.02; // Much larger!
    size_gradient.add_key(0.0, Vec3::splat(base_size));
    size_gradient.add_key(1.0, Vec3::splat(base_size * 0.8)); // Only slight shrink

    // Build the effect
    let avg_lifetime = (settings.wind_particle_lifetime_min + settings.wind_particle_lifetime_max) / 2.0;

    let writer = ExprWriter::new();

    // Random initial age so particles don't all fade in/out together
    let init_age = SetAttributeModifier::new(
        Attribute::AGE,
        writer.rand(ScalarType::Float).mul(writer.lit(avg_lifetime)).expr(),
    );

    let lifetime = writer.lit(avg_lifetime).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(particle_radius).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocitySphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(settings.wind_speed).expr(),
    };

    let spawn_rate = settings.wind_particle_count as f32 / avg_lifetime;

    info!(
        "Spawning wind particles: count={}, radius={}, height_offset={}, spawn_rate={}, size={}, lifetime={}",
        settings.wind_particle_count, radius, height_offset, spawn_rate, base_size, avg_lifetime
    );

    // Use once spawner to create all particles at startup
    let spawner = SpawnerSettings::once(CpuValue::Single(settings.wind_particle_count as f32));

    let effect = EffectAsset::new(
        32768, // capacity
        spawner,
        writer.finish(),
    )
    .with_name("wind_particles")
    .with_simulation_space(SimulationSpace::Local) // Important: particles are children of planet
    .init(init_pos)
    .init(init_vel)
    .init(init_age)
    .init(init_lifetime)
    .render(ColorOverLifetimeModifier::new(color_gradient))
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient,
        screen_space_size: false,
    })
    .render(OrientModifier::new(OrientMode::FaceCameraPosition));

    let effect_handle = effects.add(effect);

    // Spawn particle effect as child of planet
    commands.entity(planet_entity).with_children(|parent| {
        parent.spawn((
            Name::new("wind_particles"),
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
}
*/

/// Update wind particles - with Hanabi, most updates happen on GPU
/// This system just handles respawning the effect if settings change
pub fn update_wind_particles(
    mut commands: Commands,
    settings: Res<PlanetGenerationSettings>,
    particle_query: Query<(Entity, &ParticleEffect), With<WindParticle>>,
) {
    // If settings changed, despawn old effect and spawn new one
    if settings.is_changed() && settings.show_wind {
        // Despawn existing particles
        for (entity, _) in particle_query.iter() {
            commands.entity(entity).despawn();
        }

        // Respawn will happen automatically next frame via spawn_wind_particles
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
            // Wind tab deactivated - despawn particles
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
