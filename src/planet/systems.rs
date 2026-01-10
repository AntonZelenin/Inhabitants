use crate::camera::components::MainCamera;
use crate::planet::components::{ArrowEntity, CameraLerp, PlanetControls, PlanetEntity, ContinentViewMesh, PlateViewMesh};
use crate::planet::events::*;
use crate::planet::resources::*;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::{Color, LinearRgba};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::{Quat, Vec3};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::mesh::{Indices, PrimitiveTopology};
use planetgen::generator::PlanetGenerator;
use planetgen::planet::PlanetData;
use crate::mesh::helpers::arrow_mesh;

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
        planetgen::reload_config();

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

        let mut generator = PlanetGenerator::new(settings.radius);
        generator.num_plates = settings.num_plates;
        generator.num_micro_plates = settings.num_micro_plates;
        generator.seed = settings.seed;
        generator.flow_warp_freq = settings.flow_warp_freq;
        generator.flow_warp_steps = settings.flow_warp_steps;
        generator.flow_warp_step_angle = settings.flow_warp_step_angle;

        // Apply custom continent configuration from UI settings
        let continent_config = planetgen::config::ContinentConfig {
            continent_frequency: settings.continent_frequency,
            continent_amplitude: settings.continent_amplitude,
            distortion_frequency: settings.distortion_frequency,
            distortion_amplitude: settings.distortion_amplitude,
            detail_frequency: settings.detail_frequency,
            detail_amplitude: settings.detail_amplitude,
            continent_threshold: settings.continent_threshold,
            ocean_depth_amplitude: settings.ocean_depth_amplitude,
        };
        generator.with_continent_config(continent_config);

        // Apply mountain configuration from UI settings
        generator.mountain_height = settings.mountain_height;
        generator.mountain_width = settings.mountain_width;

        let planet_data = generator.generate();

        // Generate BOTH meshes (continent view and plate view)
        let continent_mesh = build_stitched_planet_mesh(&planet_data, false, settings.snow_threshold);
        let plate_mesh = build_stitched_planet_mesh(&planet_data, true, settings.snow_threshold);

        let continent_mesh_handle = meshes.add(continent_mesh);
        let plate_mesh_handle = meshes.add(plate_mesh);

        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 0.4),
            ..default()
        });


        let config = planetgen::get_config();
        let expected_zoom = settings.radius * 3.5;

        // Spawn parent planet entity with controls
        let planet_entity = commands
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0).with_rotation(current_rotation),
                GlobalTransform::default(),
                PlanetEntity,
                PlanetControls {
                    rotation: current_rotation,
                    zoom: expected_zoom,
                    min_zoom: settings.radius * 1.5,
                    max_zoom: config.generation.planet_max_radius * 3.5,
                },
            ))
            .with_children(|parent| {
                // Continent view mesh (visible by default)
                parent.spawn((
                    Mesh3d(continent_mesh_handle),
                    MeshMaterial3d(material_handle.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Visible,
                    ContinentViewMesh,
                ));

                // Plate view mesh (hidden by default)
                parent.spawn((
                    Mesh3d(plate_mesh_handle),
                    MeshMaterial3d(material_handle.clone()),
                    Transform::default(),
                    GlobalTransform::default(),
                    Visibility::Hidden,
                    PlateViewMesh,
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

fn build_stitched_planet_mesh(planet: &PlanetData, view_mode_plates: bool, snow_threshold: f32) -> Mesh {
    // Use planetgen's pure business logic to generate mesh data
    let view_mode = if view_mode_plates {
        planetgen::mesh_data::ViewMode::Plates
    } else {
        planetgen::mesh_data::ViewMode::Continents
    };
    
    let mesh_data = planetgen::mesh_data::MeshData::from_planet(planet, view_mode, snow_threshold);
    
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
