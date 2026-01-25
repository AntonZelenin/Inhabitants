use crate::camera::components::MainCamera;
use crate::mesh::helpers::arrow_mesh;
use crate::planet::components::{
    ArrowEntity, CameraLerp, ContinentView, ContinentViewMesh, OceanEntity, PlanetControls,
    PlanetEntity, PlateViewMesh, TectonicPlateView, WindParticle, WindView,
};
use crate::planet::events::*;
use crate::planet::logic;
use crate::planet::resources::*;
use crate::planet::wind_material::WindParticleMaterial;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::{Color, LinearRgba};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::{Quat, Vec3};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy_ocean::{OceanConfig, OceanMeshBuilder};
use planetgen::planet::PlanetData;
use rand::Rng;

/// Calculate age-based fade for particle transparency
/// Returns 0.0 at spawn/despawn, 1.0 at mid-life
fn calculate_age_fade(lifetime_progress: f32) -> f32 {
    if lifetime_progress < 0.2 {
        // Fade in: 0% → 100% during first 20% of life
        lifetime_progress / 0.2
    } else if lifetime_progress > 0.8 {
        // Fade out: 100% → 0% during last 20% of life
        (1.0 - lifetime_progress) / 0.2
    } else {
        // Full opacity during middle 60% of life
        1.0
    }
}

/// Create a 3D elongated particle mesh with tail (visible from all angles)
/// The particle is stretched along the Y-axis (direction of movement)
/// Creates a teardrop/bullet shape: round head (front) tapering to sharp point (back)
/// Uses only 4 rings and 6 segments (hexagon) for efficiency
fn create_cone_mesh(radius: f32, height: f32) -> Mesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Simplified: only 4 rings and 6 segments for efficiency
    let num_rings = 3;
    let segments = 4; // Hexagonal cross-section
    let angle_step = std::f32::consts::TAU / segments as f32;

    // Define radius at each ring position (creates the taper)
    // Y=0 is at origin (FRONT), Y=height is at back
    let radius_profile = [
        0.0,  // Ring 0: Front - sharp point (TIP)
        0.3,  // Ring 1: Narrow
        0.7,  // Ring 2: Mid width
        1.0,  // Ring 3: Back - widest (BASE/TAIL)
    ];

    // Generate vertices for each ring
    for ring_idx in 0..num_rings {
        let y = (ring_idx as f32 / (num_rings - 1) as f32) * height;
        let ring_radius = radius * radius_profile[ring_idx];
        let v = ring_idx as f32 / (num_rings - 1) as f32;

        if ring_radius == 0.0 {
            // Ring is a single point (the tip at front)
            positions.push([0.0, y, 0.0]);
            normals.push([0.0, -1.0, 0.0]); // Normal points backward (away from movement)
            uvs.push([0.5, v]);
        } else {
            // Generate circle of vertices
            for seg_idx in 0..segments {
                let angle = seg_idx as f32 * angle_step;
                let x = angle.cos() * ring_radius;
                let z = angle.sin() * ring_radius;

                positions.push([x, y, z]);

                // Simple normal calculation
                let radial = Vec3::new(x, 0.0, z).normalize_or_zero();
                let tangent = Vec3::Y;
                let normal = (radial + tangent * 0.5).normalize();
                normals.push([normal.x, normal.y, normal.z]);

                let u = seg_idx as f32 / segments as f32;
                uvs.push([u, v]);
            }
        }
    }

    // Generate triangles connecting the rings
    for ring_idx in 0..(num_rings - 1) {
        let current_is_point = ring_idx == 0 && radius_profile[0] == 0.0;

        if current_is_point {
            // Connect starting point to first ring of circles
            let point = 0; // Point is at index 0
            let next_ring_start = 1; // First circle ring starts at index 1

            for seg_idx in 0..segments {
                let next_a = next_ring_start + seg_idx;
                let next_b = next_ring_start + ((seg_idx + 1) % segments as usize);

                indices.push(point as u32);
                indices.push(next_a as u32);
                indices.push(next_b as u32);
            }
        } else {
            // Connect two rings with quads
            // Calculate proper start indices accounting for the single point at ring 0
            let current_ring_start = if ring_idx == 0 {
                0
            } else {
                1 + segments * (ring_idx - 1)
            };

            let next_ring_start = 1 + segments * ring_idx;

            for seg_idx in 0..segments {
                let curr_a = current_ring_start + seg_idx;
                let curr_b = current_ring_start + ((seg_idx + 1) % segments as usize);
                let next_a = next_ring_start + seg_idx;
                let next_b = next_ring_start + ((seg_idx + 1) % segments as usize);

                // Two triangles per quad
                indices.push(curr_a as u32);
                indices.push(next_a as u32);
                indices.push(curr_b as u32);

                indices.push(curr_b as u32);
                indices.push(next_a as u32);
                indices.push(next_b as u32);
            }
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

pub fn spawn_planet_on_event(
    mut commands: Commands,
    mut camera_events: MessageWriter<SetCameraPositionEvent>,
    mut events: MessageReader<GeneratePlanetEvent>,
    mut current_planet_data: ResMut<CurrentPlanetData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<PlanetGenerationSettings>,
    planet_entities: Query<Entity, With<PlanetEntity>>,
    planet_controls_query: Query<&PlanetControls, With<PlanetEntity>>,
) {
    for _ in events.read() {
        // Capture current rotation before despawning
        let current_rotation = planet_controls_query
            .iter()
            .next()
            .map(|controls| controls.rotation)
            .unwrap_or(Quat::IDENTITY);

        // Despawn existing planet entities before generating new ones
        for entity in planet_entities.iter() {
            commands.entity(entity).despawn();
        }

        let planet_data = logic::generate_planet_data(&settings);

        // PRESENTATION: Generate BOTH meshes (continent view and plate view)
        let continent_mesh = build_stitched_planet_mesh(
            &planet_data,
            false,
            settings.snow_threshold,
            settings.continent_threshold,
        );
        let plate_mesh = build_stitched_planet_mesh(
            &planet_data,
            true,
            settings.snow_threshold,
            settings.continent_threshold,
        );

        let continent_mesh_handle = meshes.add(continent_mesh);
        let plate_mesh_handle = meshes.add(plate_mesh);

        let planet_material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..default()
        });

        let config = planetgen::get_config();
        let expected_zoom = settings.radius * 3.5;

        // Spawn parent planet entity with controls
        let planet_entity = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(current_rotation),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                PlanetEntity,
                PlanetControls {
                    rotation: current_rotation,
                    zoom: expected_zoom,
                    min_zoom: settings.radius * 1.5,
                    max_zoom: config.generation.planet_max_radius * 3.5,
                },
            ))
            .with_children(|parent| {
                // Continent view mesh (visible when NOT in plate view mode)
                parent.spawn((
                    Mesh3d(continent_mesh_handle),
                    MeshMaterial3d(planet_material.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    if settings.view_mode_plates {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    },
                    ContinentViewMesh,
                    ContinentView, // Marker component
                ));

                // Plate view mesh (visible when IN plate view mode)
                parent.spawn((
                    Mesh3d(plate_mesh_handle),
                    MeshMaterial3d(planet_material.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    if settings.view_mode_plates {
                        Visibility::Visible
                    } else {
                        Visibility::Hidden
                    },
                    PlateViewMesh,
                    TectonicPlateView, // Marker component
                ));
            })
            .id();

        camera_events.write(SetCameraPositionEvent {
            position: Vec3::new(0.0, 0.0, expected_zoom),
        });

        if settings.show_arrows {
            spawn_plate_direction_arrows(
                &mut commands,
                &mut meshes,
                &mut materials,
                &planet_data,
                planet_entity,
            );
        }

        // Spawn ocean sphere at sea level (only visible in continent view mode)
        if settings.show_ocean {
            spawn_ocean(
                &mut commands,
                &mut meshes,
                &mut materials,
                &settings,
                planet_entity,
                settings.view_mode_plates, // Pass view mode to control initial visibility
            );
        }

        // Store planet data after using it for generation
        current_planet_data.planet_data = Some(planet_data);
    }
}

pub fn handle_arrow_toggle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: MessageReader<ToggleArrowsEvent>,
    arrow_entities: Query<Entity, With<ArrowEntity>>,
    planet_entities: Query<Entity, (With<PlanetEntity>, With<PlanetControls>)>,
    current_planet_data: Res<CurrentPlanetData>,
) {
    for event in events.read() {
        if event.show_arrows {
            // Only spawn arrows if we have planet data and no arrows currently exist
            if let Some(ref planet_data) = current_planet_data.planet_data {
                if arrow_entities.is_empty() {
                    if let Ok(planet_entity) = planet_entities.single() {
                        spawn_plate_direction_arrows(
                            &mut commands,
                            &mut meshes,
                            &mut materials,
                            planet_data,
                            planet_entity,
                        );
                    }
                }
            }
        } else {
            // Despawn all existing arrows
            for entity in arrow_entities.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn build_stitched_planet_mesh(
    planet: &PlanetData,
    view_mode_plates: bool,
    snow_threshold: f32,
    continent_threshold: f32,
) -> Mesh {
    // Use planetgen's pure business logic to generate mesh data
    let view_mode = if view_mode_plates {
        planetgen::mesh_data::ViewMode::Plates
    } else {
        planetgen::mesh_data::ViewMode::Continents
    };

    let mesh_data = planetgen::mesh_data::MeshData::from_planet(
        planet,
        view_mode,
        snow_threshold,
        continent_threshold,
    );

    // Convert to Bevy mesh (thin presentation layer)
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, mesh_data.colors);
    mesh.insert_indices(Indices::U32(mesh_data.indices));
    mesh
}

fn spawn_plate_direction_arrows(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    planet: &PlanetData,
    planet_entity: Entity,
) {
    // Use planetgen's pure business logic to calculate arrow data
    let arrow_data = planetgen::arrows::calculate_plate_arrows(planet);

    // Prepare Bevy resources (presentation layer)
    let arrow_mesh = arrow_mesh();
    let arrow_mesh_handle = meshes.add(arrow_mesh);
    let arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 0.4),
        emissive: LinearRgba::BLUE,
        ..default()
    });

    // Spawn arrow entities from calculated data
    for arrow in arrow_data {
        let arrow_entity = commands
            .spawn((
                Mesh3d(arrow_mesh_handle.clone()),
                MeshMaterial3d(arrow_material.clone()),
                Transform::from_translation(arrow.position)
                    .with_rotation(arrow.rotation)
                    .with_scale(Vec3::splat(arrow.scale)),
                GlobalTransform::default(),
                ArrowEntity,
            ))
            .id();

        // Make the arrow a child of the planet entity
        commands.entity(planet_entity).add_child(arrow_entity);
    }
}

fn spawn_ocean(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    settings: &PlanetGenerationSettings,
    planet_entity: Entity,
    view_mode_plates: bool,
) {
    let ocean_config = OceanConfig {
        sea_level: settings.radius,
        grid_size: 256, // Much higher detail for smooth appearance
        wave_amplitude: settings.ocean_wave_amplitude,
        wave_frequency: settings.ocean_wave_frequency,
        wave_speed: settings.ocean_wave_speed,
        ocean_color: Color::srgba(0.02, 0.15, 0.35, 0.9), // Deep blue with some transparency
    };

    let ocean = OceanMeshBuilder::new(ocean_config).with_time(0.0).build();

    let ocean_entity = commands
        .spawn((
            Mesh3d(meshes.add(ocean.mesh)),
            MeshMaterial3d(materials.add(ocean.material)),
            Transform::default(),
            GlobalTransform::default(),
            if view_mode_plates {
                Visibility::Hidden
            } else {
                Visibility::Visible
            },
            OceanEntity,
            ContinentView, // Marker component - will be toggled by view mode system
        ))
        .id();

    // Make the ocean a child of the planet entity so it rotates with the planet
    commands.entity(planet_entity).add_child(ocean_entity);
}

pub fn planet_control(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    mut planet_query: Query<
        (&mut Transform, &mut PlanetControls),
        (With<PlanetEntity>, With<PlanetControls>),
    >,
    mut camera_query: Query<&mut CameraLerp, With<Camera3d>>,
    windows: Query<&Window>,
) {
    if let Ok((mut planet_transform, mut controls)) = planet_query.single_mut() {
        if let Ok(mut camera_lerp) = camera_query.single_mut() {
            let window = windows.single().unwrap();
            let cursor_position = window.cursor_position();

            // Check if cursor is over UI (right 25% of screen)
            let is_over_ui = if let Some(cursor_pos) = cursor_position {
                cursor_pos.x > window.width() * 0.75
            } else {
                false
            };

            // Handle mouse dragging for planet rotation (only Y-axis) - only if not over UI
            if mouse_input.pressed(MouseButton::Left) && !is_over_ui {
                for motion in mouse_motion.read() {
                    let sensitivity = 0.002 * (controls.zoom / 60.0);
                    let yaw = Quat::from_rotation_y(motion.delta.x * sensitivity);

                    controls.rotation = controls.rotation * yaw;
                    planet_transform.rotation = controls.rotation;
                }
            }

            // Handle mouse wheel for zoom - only if not over UI
            if !is_over_ui {
                for wheel in mouse_wheel.read() {
                    controls.zoom -= wheel.y * 2.0;
                    controls.zoom = controls.zoom.clamp(controls.min_zoom, controls.max_zoom);

                    // Recompute composition offsets from current distance
                    let camera_x_offset = controls.zoom * 0.25;
                    let look_at_x_offset = controls.zoom * 0.15;

                    camera_lerp.target_position = Vec3::new(camera_x_offset, 0.0, controls.zoom);
                    camera_lerp.target_look_at = Vec3::new(look_at_x_offset, 0.0, 0.0);
                    camera_lerp.is_lerping = true;
                }
            }
        }
    }
}

pub fn smooth_camera_movement(
    time: Res<Time>,
    mut camera_query: Query<(&mut Transform, &mut CameraLerp), With<Camera3d>>,
) {
    if let Ok((mut camera_transform, mut camera_lerp)) = camera_query.single_mut() {
        if camera_lerp.is_lerping {
            let dt = time.delta_secs();
            let lerp_factor = (camera_lerp.lerp_speed * dt).min(1.0);

            // Lerp position directly toward target
            camera_transform.translation = camera_transform
                .translation
                .lerp(camera_lerp.target_position, lerp_factor);

            // Lerp look_at independently toward target to avoid sudden direction changes
            camera_lerp.current_look_at = camera_lerp
                .current_look_at
                .lerp(camera_lerp.target_look_at, lerp_factor);

            // Apply the smoothed look_at every frame
            camera_transform.look_at(camera_lerp.current_look_at, Vec3::Y);

            // Stop when both position and look_at are effectively at target
            let pos_dist = camera_transform
                .translation
                .distance(camera_lerp.target_position);
            let look_dist = camera_lerp
                .current_look_at
                .distance(camera_lerp.target_look_at);

            if pos_dist < 0.001 && look_dist < 0.001 {
                // Snap the last tiny epsilon to avoid drift (imperceptible)
                camera_transform.translation = camera_lerp.target_position;
                camera_lerp.current_look_at = camera_lerp.target_look_at;
                camera_transform.look_at(camera_lerp.current_look_at, Vec3::Y);
                camera_lerp.is_lerping = false;
            }
        }
    }
}

pub fn handle_camera_position_events(
    mut events: MessageReader<SetCameraPositionEvent>,
    mut camera_query: Query<&mut CameraLerp, With<MainCamera>>,
) {
    for event in events.read() {
        if let Ok(mut camera_lerp) = camera_query.single_mut() {
            let distance = event.position.z.max(0.0);

            // Recompute offsets from current distance to keep composition stable
            let camera_x_offset = distance * 0.25;
            let look_at_x_offset = distance * 0.15;

            camera_lerp.target_position = Vec3::new(camera_x_offset, event.position.y, distance);
            camera_lerp.target_look_at = Vec3::new(look_at_x_offset, 0.0, 0.0);

            // Immediately align the current look to new target to prevent sideways motion on regen
            camera_lerp.current_look_at = camera_lerp.target_look_at;

            // Helper values (not used for zoom path now, but kept for clarity)
            camera_lerp.pivot = camera_lerp.target_look_at;
            camera_lerp.dir = Vec3::Z;

            camera_lerp.is_lerping = true;
        }
    }
}

pub fn handle_generate_new_seed(
    mut events: MessageReader<GenerateNewSeedEvent>,
    mut settings: ResMut<PlanetGenerationSettings>,
    mut settings_changed_events: MessageWriter<SettingsChanged>,
) {
    for _ in events.read() {
        // Generate a new 8-bit user seed using planetgen
        let new_user_seed = planetgen::tools::generate_seed8();

        // Update both user seed and the expanded 64-bit seed
        settings.user_seed = new_user_seed;
        settings.seed = planetgen::tools::expand_seed64(new_user_seed);
        settings_changed_events.write(SettingsChanged);
    }
}

/// Spawn wind particles when wind visualization is enabled
pub fn spawn_wind_particles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<WindParticleMaterial>>,
    settings: Res<PlanetGenerationSettings>,
    existing_particles: Query<Entity, With<WindParticle>>,
    planet_query: Query<Entity, With<PlanetEntity>>,
) {
    // Only spawn if wind is enabled and we don't have particles yet
    if !settings.show_wind || !existing_particles.is_empty() {
        return;
    }

    let Ok(planet_entity) = planet_query.single() else {
        return;
    };

    let radius = settings.radius;

    // Create simplified 3D particle mesh (visible from all angles)
    // Only 4 rings × 6 segments = ~19 vertices, ~24 triangles per particle
    let cone_height = settings.wind_particle_mesh_size * 2.0;
    let cone_radius = settings.wind_particle_mesh_size * 0.5;
    let base_mesh = create_cone_mesh(cone_radius, cone_height);

    // Create single shared material (efficient batching!)
    let particle_material_handle = materials.add(WindParticleMaterial::default());

    let mut rng = rand::rng();

    // Spawn particles uniformly distributed on sphere
    for i in 0..settings.wind_particle_count {
        // Use golden angle spiral for uniform distribution
        let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let theta = 2.0 * std::f32::consts::PI * i as f32 / golden_ratio;
        let phi = (1.0 - 2.0 * (i as f32 + 0.5) / settings.wind_particle_count as f32).acos();

        let x = phi.sin() * theta.cos();
        let y = phi.sin() * theta.sin();
        let z = phi.cos();

        let position = Vec3::new(x, y, z).normalize();

        // Initial velocity based on latitude zone
        let velocity = calculate_wind_velocity(&position, settings.wind_speed);

        // Randomize lifetime using config values
        let lifetime = rng
            .random_range(settings.wind_particle_lifetime_min..settings.wind_particle_lifetime_max);

        // Randomize initial age to prevent synchronized despawning
        let initial_age = rng.random_range(0.0..lifetime);

        // Calculate age-based fade for vertex colors
        let lifetime_progress = initial_age / lifetime;
        let age_fade = calculate_age_fade(lifetime_progress);

        // Create mesh with vertex colors (white RGB, alpha = age_fade)
        let mut particle_mesh = base_mesh.clone();
        let vertex_count = particle_mesh.count_vertices();
        let colors: Vec<[f32; 4]> = vec![[1.0, 1.0, 1.0, age_fade]; vertex_count];
        particle_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        let particle_mesh_handle = meshes.add(particle_mesh);

        commands.entity(planet_entity).with_children(|parent| {
            parent.spawn((
                Mesh3d(particle_mesh_handle),
                MeshMaterial3d(particle_material_handle.clone()),
                Transform::from_translation(
                    position * (radius + settings.wind_particle_height_offset),
                ),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                WindParticle {
                    position,
                    velocity,
                    age: initial_age,
                    lifetime,
                    particle_id: i as u32,
                    respawn_count: 0,
                },
                WindView,
            ));
        });
    }
}

/// Calculate wind velocity based on latitude zone (Hadley, Ferrel, and Polar cells)
/// - 0-30° latitude: wind moves toward equator
/// - 30-60° latitude: wind moves toward pole
/// - 60-90° latitude: wind moves toward equator
fn calculate_wind_velocity(position: &Vec3, wind_speed: f32) -> Vec3 {
    let up = Vec3::Y;

    // Calculate latitude in radians (-PI/2 to PI/2)
    let latitude = position.y.asin();
    let abs_latitude_deg = latitude.abs().to_degrees();

    // Calculate north/south direction (tangent to sphere, perpendicular to equator)
    // Positive = toward north pole, negative = toward south pole
    // Fix: was using double cross product which inverted the direction
    let northward = position.cross(up).cross(*position).normalize();

    // Determine meridional (north-south) direction based on latitude zone
    let meridional_direction = if abs_latitude_deg < 30.0 {
        // 0-30°: Trade winds - move toward equator
        if latitude > 0.0 { -1.0 } else { 1.0 }
    } else if abs_latitude_deg < 60.0 {
        // 30-60°: Westerlies - move toward pole
        if latitude > 0.0 { 1.0 } else { -1.0 }
    } else {
        // 60-90°: Polar easterlies - move toward equator
        if latitude > 0.0 { -1.0 } else { 1.0 }
    };

    // Move perpendicular to equator only (meridional flow)
    let velocity = northward * (meridional_direction * wind_speed);

    // Keep velocity tangent to sphere
    velocity - *position * position.dot(velocity)
}

/// Update wind particle positions
pub fn update_wind_particles(
    time: Res<Time>,
    settings: Res<PlanetGenerationSettings>,
    mut particles: Query<(&mut Transform, &mut WindParticle, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let dt = time.delta_secs();
    let radius = settings.radius;
    let mut rng = rand::rng();

    for (mut transform, mut particle, mesh_handle) in particles.iter_mut() {
        // Age the particle
        particle.age += dt;

        // Update vertex colors for age-based fade
        let lifetime_progress = (particle.age / particle.lifetime).clamp(0.0, 1.0);
        let age_fade = calculate_age_fade(lifetime_progress);

        if let Some(mesh) = meshes.get_mut(&mesh_handle.0) {
            let vertex_count = mesh.count_vertices();
            let colors: Vec<[f32; 4]> = vec![[1.0, 1.0, 1.0, age_fade]; vertex_count];
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        }
        particle.age += dt;

        // Respawn if too old
        if particle.age > particle.lifetime {
            // Increment respawn counter for unique randomization
            particle.respawn_count += 1;

            // Randomly position on sphere
            let theta = rng.random_range(0.0..std::f32::consts::TAU);
            let z = rng.random_range(-1.0_f32..1.0_f32);
            let r = (1.0 - z * z).sqrt();

            let x = r * theta.cos();
            let y = r * theta.sin();

            particle.position = Vec3::new(x, y, z).normalize();
            particle.age = 0.0;

            // Randomize next lifetime using config values
            particle.lifetime = rng.random_range(
                settings.wind_particle_lifetime_min..settings.wind_particle_lifetime_max,
            );

            // Calculate velocity based on latitude zone
            particle.velocity = calculate_wind_velocity(&particle.position, settings.wind_speed);
        }

        // Move particle along velocity (on sphere surface)
        let displacement = particle.velocity * dt;

        // Update position on sphere
        particle.position = (particle.position + displacement).normalize();

        // Calculate target velocity based on new latitude zone
        let target_velocity = calculate_wind_velocity(&particle.position, settings.wind_speed);

        // Smoothly interpolate current velocity toward target velocity
        // Preserve speed to prevent particles from slowing down at boundaries
        let current_speed = particle.velocity.length();
        let turn_rate = settings.wind_turn_rate.clamp(0.01, 1.0);

        // Interpolate direction
        let new_velocity = particle.velocity.lerp(target_velocity, turn_rate);

        // Restore original speed to maintain momentum
        particle.velocity = if new_velocity.length() > 0.001 {
            new_velocity.normalize() * current_speed
        } else {
            target_velocity // Fallback to target if velocity becomes zero
        };

        // Ensure velocity stays tangent to sphere
        particle.velocity = particle.velocity - particle.position * particle.position.dot(particle.velocity);

        // Update transform with velocity-based stretching for trail effect
        let velocity_length = particle.velocity.length();
        // Use config-based stretch multiplier for trail visibility
        let stretch_scale = 1.0
            + (velocity_length
                * settings.wind_trail_length
                * settings.wind_particle_stretch_multiplier);

        // Calculate rotation to align with velocity direction
        // Mesh Y-axis points from tip (Y=0) to tail (Y=height)
        // We want tail to trail BEHIND the movement, so Y-axis should point OPPOSITE to velocity
        let velocity_dir = particle.velocity.normalize_or_zero();
        let rotation = if velocity_dir.length() > 0.001 {
            // Normal points radially outward (this will be Z-axis of the mesh)
            let normal = particle.position.normalize();

            let backward = -velocity_dir;

            // tangent direction on sphere (trailing)
            let mut forward = (backward - normal * backward.dot(normal)).normalize_or_zero();

            if forward.length() < 0.001 {
                let arbitrary = if normal.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
                forward = (arbitrary - normal * normal.dot(arbitrary)).normalize();
            }

            // X = Y × Z  (right-handed)
            let right = forward.cross(normal).normalize();

            // (optional but recommended) re-orthonormalise Y
            let forward = normal.cross(right).normalize();

            Quat::from_mat3(&Mat3::from_cols(right, forward, normal))
        } else {
            // Fallback: just point radially outward
            let normal = particle.position.normalize();
            Quat::from_rotation_arc(Vec3::Z, normal)
        };

        transform.translation = particle.position * (radius + settings.wind_particle_height_offset);
        transform.rotation = rotation;
        transform.scale = Vec3::new(1.0, stretch_scale, 1.0); // Stretch along Y (forward)
    }
}

/// Despawn wind particles when wind visualization is disabled
pub fn despawn_wind_particles(
    mut commands: Commands,
    settings: Res<PlanetGenerationSettings>,
    particles: Query<Entity, With<WindParticle>>,
) {
    if !settings.show_wind {
        for entity in particles.iter() {
            commands.entity(entity).despawn();
        }
    }
}
