// Wind particle systems using Hanabi

use crate::planet::components::PlanetEntity;
use crate::planet::events::WindTabActiveEvent;
use crate::planet::resources::PlanetGenerationSettings;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

/// Marker component for wind particle effect
#[derive(Component)]
pub struct WindParticleEffect;

/// Handle wind tab activation/deactivation
pub fn handle_wind_tab_events(
    mut wind_tab_events: MessageReader<WindTabActiveEvent>,
    mut planet_settings: ResMut<PlanetGenerationSettings>,
    existing_effects: Query<Entity, With<WindParticleEffect>>,
    mut commands: Commands,
    mut effects: ResMut<Assets<EffectAsset>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
) {
    for event in wind_tab_events.read() {
        planet_settings.show_wind = event.active;

        if event.active && existing_effects.is_empty() {
            // Spawn hanabi particle effect
            if let Some(planet_entity) = planet_query.iter().next() {
                let particle_count = planet_settings.wind_particle_count;
                info!("Spawning {} static wind particles with Hanabi", particle_count);

                let writer = ExprWriter::new();

                // Random lifetime between 3 and 7 seconds per particle
                let init_lifetime = SetAttributeModifier::new(
                    Attribute::LIFETIME,
                    writer.lit(3.0).uniform(writer.lit(7.0)).expr(),
                );

                // Random initial age so particles fade in/out at different times
                let init_age = SetAttributeModifier::new(
                    Attribute::AGE,
                    writer.lit(0.0).uniform(writer.lit(7.0)).expr(),
                );

                // Spawn particles distributed around sphere
                let sphere_radius = planet_settings.radius + planet_settings.wind_particle_height_offset;
                let init_pos_sphere = SetPositionSphereModifier {
                    center: writer.lit(Vec3::ZERO).expr(),
                    radius: writer.lit(sphere_radius).expr(),
                    dimension: ShapeDimension::Surface,
                };

                // Light gray/white color that fades in and out
                let mut color_gradient = bevy_hanabi::Gradient::new();
                color_gradient.add_key(0.0, Vec4::new(0.9, 0.9, 0.9, 0.0)); // Fade in from transparent
                color_gradient.add_key(0.1, Vec4::new(0.9, 0.9, 0.9, 1.0)); // Fully visible
                color_gradient.add_key(0.9, Vec4::new(0.9, 0.9, 0.9, 1.0)); // Stay visible
                color_gradient.add_key(1.0, Vec4::new(0.9, 0.9, 0.9, 0.0)); // Fade out to transparent

                let mut size_gradient = bevy_hanabi::Gradient::new();
                size_gradient.add_key(0.0, Vec3::splat(0.4));
                size_gradient.add_key(1.0, Vec3::splat(0.4));

                // Create TWO effects to achieve the desired behavior:
                // 1. Initial batch: spawn particle_count particles immediately with random lifetimes
                // 2. Continuous replacement: spawn individual particles at a rate to replace dying ones

                // Effect 1: Initial batch with random lifetimes (spawns once immediately)
                let spawner_initial = SpawnerSettings::once((particle_count as f32).into());

                let effect_initial = effects.add(
                    EffectAsset::new(
                        particle_count as u32,
                        spawner_initial,
                        writer.finish()
                    )
                        .with_name("wind_particles_initial")
                        .init(init_pos_sphere.clone())
                        .init(init_age.clone()) // Random age 0-7s
                        .init(init_lifetime.clone()) // Random lifetime 3-7s
                        .render(ColorOverLifetimeModifier {
                            gradient: color_gradient.clone(),
                            blend: ColorBlendMode::Modulate,
                            mask: ColorBlendMask::RGBA,
                        })
                        .render(SizeOverLifetimeModifier {
                            gradient: size_gradient.clone(),
                            screen_space_size: false,
                        }),
                );

                // Effect 2: Continuous spawner to replace dying particles
                let writer2 = ExprWriter::new();

                let init_lifetime2 = SetAttributeModifier::new(
                    Attribute::LIFETIME,
                    writer2.lit(5.0).expr(), // Constant 5s lifetime for replacements
                );

                let init_age2 = SetAttributeModifier::new(
                    Attribute::AGE,
                    writer2.lit(0.0).expr(), // Start at age 0
                );

                let init_pos_sphere2 = SetPositionSphereModifier {
                    center: writer2.lit(Vec3::ZERO).expr(),
                    radius: writer2.lit(sphere_radius).expr(),
                    dimension: ShapeDimension::Surface,
                };

                let spawner_continuous = SpawnerSettings::rate((particle_count as f32 / 5.0).into());

                let effect_continuous = effects.add(
                    EffectAsset::new(
                        particle_count as u32,
                        spawner_continuous,
                        writer2.finish()
                    )
                        .with_name("wind_particles_continuous")
                        .init(init_pos_sphere2)
                        .init(init_age2)
                        .init(init_lifetime2)
                        .render(ColorOverLifetimeModifier {
                            gradient: color_gradient,
                            blend: ColorBlendMode::Modulate,
                            mask: ColorBlendMask::RGBA,
                        })
                        .render(SizeOverLifetimeModifier {
                            gradient: size_gradient,
                            screen_space_size: false,
                        }),
                );

                commands.entity(planet_entity).with_children(|parent| {
                    parent.spawn((
                        ParticleEffect::new(effect_initial),
                        WindParticleEffect,
                    ));
                    parent.spawn((
                        ParticleEffect::new(effect_continuous),
                        WindParticleEffect,
                    ));
                });
            }
        } else if !event.active {
            // Despawn wind particles when switching away from wind tab
            for entity in existing_effects.iter() {
                commands.entity(entity).despawn();
            }
            info!("Wind particles despawned");
        }
    }
}
