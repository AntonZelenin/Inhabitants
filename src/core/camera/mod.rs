pub(crate) mod components;

use crate::core::camera::components::{MainCamera, MainCameraTarget};
use crate::core::state::GameState;
use crate::planet::components::CameraLerp;
use crate::planet::events::SetCameraPositionEvent;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use std::f32::consts::PI;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainCamera>()
            .register_type::<MainCameraTarget>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                PostUpdate,
                camera_control.run_if(in_state(GameState::InGame)),
            )
            .add_systems(Update, handle_camera_position_events);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 60.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainCamera,
        CameraLerp {
            target_position: Vec3::new(0.0, 0.0, 60.0),
            target_look_at: Vec3::ZERO,
            current_look_at: Vec3::ZERO,
            pivot: Vec3::ZERO,
            dir: Vec3::Z,
            lerp_speed: 3.0,
            is_lerping: false,
        },
    ));

    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight::default(),
    ));

    info!("Camera spawned");
}

fn camera_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time>,
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<MainCameraTarget>)>,
) {
    let dt = time.delta().as_secs_f32();
    let mut transform = camera_q.single_mut().unwrap();

    let mut speed = 5.0;
    if keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        speed *= 5.0;
    }

    let forward = transform.rotation.mul_vec3(Vec3::new(0.0, 0.0, -1.0));
    let right = transform.rotation.mul_vec3(Vec3::new(1.0, 0.0, 0.0));
    let mut dir = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        dir += forward;
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        dir -= forward;
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if dir.length_squared() > 0.0 {
        transform.translation += dir.normalize() * speed * dt;
    }

    if mouse_input.pressed(MouseButton::Right) {
        for ev in mouse_motion.read() {
            let yaw = Quat::from_rotation_y(-ev.delta.x * 0.002);
            let pitch = Quat::from_rotation_x(-ev.delta.y * 0.002);
            transform.rotation = yaw * transform.rotation * pitch;
        }
    }

    for ev in mouse_wheel.read() {
        transform.translation += forward * ev.y * 0.5;
    }
}

fn handle_camera_position_events(
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