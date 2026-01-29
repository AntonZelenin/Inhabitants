// Wind particle systems using Hanabi

use crate::planet::components::PlanetEntity;
use crate::planet::events::{PlanetSpawnedEvent, WindTabActiveEvent};
use crate::planet::resources::{CurrentPlanetData, PlanetGenerationSettings};
use crate::planet::wind::components::{WindMapMarker, WindTextureHandle};
use bevy::prelude::*;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_hanabi::prelude::*;

/// Marker component for wind particle effect
#[derive(Component)]
pub struct WindParticleEffect;

/// Clean up old wind texture when a new planet is generated
pub fn cleanup_wind_texture_on_new_planet(
    mut planet_spawned_events: MessageReader<PlanetSpawnedEvent>,
    existing_wind_map: Query<Entity, With<WindMapMarker>>,
    mut commands: Commands,
) {
    for _event in planet_spawned_events.read() {
        // Despawn the old wind texture entity if it exists
        for entity in existing_wind_map.iter() {
            commands.entity(entity).despawn();
            info!("Cleaned up old wind texture for new planet");
        }
    }
}

/// Creates the wind field texture when the wind tab is activated
/// This system ensures the wind texture is generated only once and persists
pub fn ensure_wind_texture(
    mut wind_tab_events: MessageReader<WindTabActiveEvent>,
    existing_wind_map: Query<Entity, With<WindMapMarker>>,
    mut planet_data_res: ResMut<CurrentPlanetData>,
    planet_settings: Res<PlanetGenerationSettings>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    for event in wind_tab_events.read() {
        if event.active && existing_wind_map.is_empty() {
            info!("Wind tab activated - generating wind field texture");

            // Get or generate the wind field
            if let Some(planet_data) = planet_data_res.planet_data.as_mut() {
                let wind_speed = planet_settings.wind_speed;

                // Get grid size before borrowing for wind map
                let grid_size = planet_data.face_grid_size;

                // Generate wind field if needed and get reference
                let wind_field = planet_data.get_wind_map(wind_speed);

                // Create a cube-sphere atlas texture
                // Layout: 6 faces arranged horizontally or in a cross pattern
                // For simplicity, we'll use horizontal layout: [Face0][Face1][Face2][Face3][Face4][Face5]
                let atlas_width = grid_size * 6;
                let atlas_height = grid_size;

                // Each pixel stores RG (wind x, y components) as normalized values
                let mut texture_data = vec![0u8; atlas_width * atlas_height * 4]; // RGBA

                for (face_idx, face) in wind_field.iter().enumerate() {
                    let face_offset_x = face_idx * grid_size;

                    for y in 0..grid_size {
                        for x in 0..grid_size {
                            let wind_vector = face.vectors[y][x];

                            // Convert wind vector to normalized [0, 1] range
                            // Assuming wind speeds are roughly in range [-10, 10]
                            let max_wind = 10.0;
                            let r = ((wind_vector.x / max_wind + 1.0) * 0.5 * 255.0).clamp(0.0, 255.0) as u8;
                            let g = ((wind_vector.y / max_wind + 1.0) * 0.5 * 255.0).clamp(0.0, 255.0) as u8;

                            let pixel_x = face_offset_x + x;
                            let pixel_idx = (y * atlas_width + pixel_x) * 4;

                            texture_data[pixel_idx] = r;
                            texture_data[pixel_idx + 1] = g;
                            texture_data[pixel_idx + 2] = 0; // Unused
                            texture_data[pixel_idx + 3] = 255; // Alpha
                        }
                    }
                }

                // Create the texture
                let wind_texture = Image::new(
                    Extent3d {
                        width: atlas_width as u32,
                        height: atlas_height as u32,
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    texture_data,
                    TextureFormat::Rgba8Unorm,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                );

                let wind_texture_handle = images.add(wind_texture);

                // Spawn entity with wind texture and marker
                commands.spawn((
                    WindTextureHandle(wind_texture_handle),
                    WindMapMarker,
                    Name::new("WindFieldTexture"),
                ));

                info!("Wind field texture created: {}x{} atlas with {} faces",
                    atlas_width, atlas_height, 6);
            } else {
                warn!("Cannot create wind texture: no planet data available");
            }
        }
    }
}

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

                // 1 create a simple texture, where wind blows in the same direction
                // 2 apply this texture to the particles as a POC
                // 3 generate a more complex texture in multiple passes
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
