use crate::core::camera::components::MainCamera;
use crate::helpers::mesh::arrow_mesh;
use crate::planet::components::{ArrowEntity, CameraLerp, PlanetControls, PlanetEntity};
use crate::planet::constants::PLANET_MAX_RADIUS;
use crate::planet::events::{GeneratePlanetEvent, SetCameraPositionEvent, ToggleArrowsEvent};
use crate::planet::resources::*;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::{Color, LinearRgba};
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::{Quat, Vec3};
use bevy::pbr::{MeshMaterial3d, StandardMaterial};
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use planetgen::prelude::*;
use std::collections::HashMap;

pub fn spawn_planet_on_event(
    mut commands: Commands,
    mut camera_events: EventWriter<SetCameraPositionEvent>,
    mut events: EventReader<GeneratePlanetEvent>,
    mut current_planet_data: ResMut<CurrentPlanetData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<PlanetGenerationSettings>,
    planet_entities: Query<Entity, With<PlanetEntity>>,
) {
    for _ in events.read() {
        // Despawn existing planet entities before generating new ones
        for entity in planet_entities.iter() {
            commands.entity(entity).despawn();
        }

        let planet_data = generate((&*settings).into());

        // Store planet data for arrow generation (move instead of clone)
        let mesh = build_stitched_planet_mesh(&planet_data);
        let mesh_handle = meshes.add(mesh);

        let material_handle = materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.8, 0.4),
            ..default()
        });

        let expected_zoom = settings.radius * 3.0;

        let planet_entity = commands
            .spawn((
                Mesh3d(mesh_handle),
                MeshMaterial3d(material_handle),
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                PlanetEntity,
                PlanetControls {
                    rotation: Quat::IDENTITY,
                    zoom: expected_zoom,
                    min_zoom: settings.radius * 1.5,
                    max_zoom: PLANET_MAX_RADIUS * 3.5,
                },
            ))
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
    mut events: EventReader<ToggleArrowsEvent>,
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

fn build_stitched_planet_mesh(planet: &PlanetData) -> Mesh {
    let size = planet.face_grid_size;
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut dir_map: HashMap<(i32, i32, i32), u32> = HashMap::new();
    let mut vertex_indices = vec![vec![vec![0u32; size]; size]; 6];
    let mut next_index = 0u32;

    let quant_scale = (size - 1) as f32;

    for (face_idx, face) in planet.faces.iter().enumerate() {
        for y in 0..size {
            let v = (y as f32 / (size - 1) as f32) * 2.0 - 1.0;
            for x in 0..size {
                let u = (x as f32 / (size - 1) as f32) * 2.0 - 1.0;
                let (nx, ny, nz) = cube_face_point(face_idx, u, v);
                let dir = Vec3::new(nx, ny, nz).normalize();

                let key = (
                    (dir.x * quant_scale).round() as i32,
                    (dir.y * quant_scale).round() as i32,
                    (dir.z * quant_scale).round() as i32,
                );

                let idx = *dir_map.entry(key).or_insert_with(|| {
                    let height = face.heightmap[y][x];
                    let radius = planet.radius + height;
                    let pos = dir * radius;
                    positions.push([pos.x, pos.y, pos.z]);

                    let plate_id = planet.plate_map[face_idx][y][x];
                    let color = planet.plates[plate_id].debug_color;
                    colors.push(color);

                    let i = next_index;
                    next_index += 1;
                    i
                });

                vertex_indices[face_idx][y][x] = idx;
            }
        }
    }

    for face_idx in 0..6 {
        for y in 0..(size - 1) {
            for x in 0..(size - 1) {
                let i0 = vertex_indices[face_idx][y][x];
                let i1 = vertex_indices[face_idx][y][x + 1];
                let i2 = vertex_indices[face_idx][y + 1][x];
                let i3 = vertex_indices[face_idx][y + 1][x + 1];
                indices.extend_from_slice(&[i0, i1, i2, i1, i3, i2]);
            }
        }
    }

    let normals: Vec<[f32; 3]> = positions
        .iter()
        .map(|p| Vec3::from(*p).normalize().to_array())
        .collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn spawn_plate_direction_arrows(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    planet: &PlanetData,
    planet_entity: Entity,
) {
    let arrow_mesh = arrow_mesh();
    let arrow_mesh_handle = meshes.add(arrow_mesh);
    let arrow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.8, 0.4),
        emissive: LinearRgba::BLUE,
        ..default()
    });

    // Calculate the scale factor (10% of planet radius)
    let arrow_scale = planet.radius * 0.2;

    // For each plate, calculate its center position
    for (plate_idx, plate) in planet.plates.iter().enumerate() {
        // Calculate center position by finding the average position of all cells belonging to this plate
        let mut center = Vec3::ZERO;
        let mut count = 0;

        // Iterate through all faces and find cells belonging to this plate
        for (face_idx, face) in planet.faces.iter().enumerate() {
            for y in 0..planet.face_grid_size {
                for x in 0..planet.face_grid_size {
                    if planet.plate_map[face_idx][y][x] == plate_idx {
                        // Convert grid position to 3D position
                        let u = (x as f32 / (planet.face_grid_size - 1) as f32) * 2.0 - 1.0;
                        let v = (y as f32 / (planet.face_grid_size - 1) as f32) * 2.0 - 1.0;
                        let (nx, ny, nz) = cube_face_point(face_idx, u, v);
                        let dir = Vec3::new(nx, ny, nz).normalize();
                        let height = face.heightmap[y][x];
                        let pos = dir * (planet.radius + height);

                        center += pos;
                        count += 1;
                    }
                }
            }
        }

        // Calculate average position if we found any cells
        if count > 0 {
            center /= count as f32;
            // Normalize to the planet radius and add a small offset
            center = center.normalize() * (planet.radius + 1.0);

            // Get the movement direction of the plate
            let direction =
                Vec3::new(plate.direction.x, plate.direction.y, plate.direction.z).normalize();

            // Get the surface normal at this position (pointing outward from center)
            let surface_normal = center.normalize();

            // Project the plate direction onto the tangent plane at this surface point
            // This removes the component of the direction that points toward/away from the center
            let tangent_direction =
                (direction - surface_normal * direction.dot(surface_normal)).normalize();

            // Calculate rotation to point in the tangent direction
            let default_direction = Vec3::Z;
            let rotation = Quat::from_rotation_arc(default_direction, tangent_direction);

            let arrow_entity = commands
                .spawn((
                    Mesh3d(arrow_mesh_handle.clone()),
                    MeshMaterial3d(arrow_material.clone()),
                    Transform::from_translation(center)
                        .with_rotation(rotation)
                        .with_scale(Vec3::splat(arrow_scale)),
                    GlobalTransform::default(),
                    ArrowEntity,
                ))
                .id();

            // Make the arrow a child of the planet entity
            commands.entity(planet_entity).add_child(arrow_entity);
        }
    }
}

pub fn planet_control(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
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
    mut events: EventReader<SetCameraPositionEvent>,
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