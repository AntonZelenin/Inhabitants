use crate::camera::components::{MainCamera, MainCameraTarget};
use crate::camera::logic::{calculate_camera_transform, CameraInput};
use crate::planet::components::CameraLerp;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::log::info;
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::light::DirectionalLight;
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn spawn_camera(mut commands: Commands) {
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

pub fn camera_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut mouse_wheel: MessageReader<MouseWheel>,
    time: Res<Time>,
    mut camera_q: Query<&mut Transform, (With<MainCamera>, Without<MainCameraTarget>)>,
) {
    // Read ECS state
    let dt = time.delta().as_secs_f32();
    let mut transform = camera_q.single_mut().unwrap();

    // Collect mouse motion delta
    let mut total_mouse_delta = Vec3::ZERO;
    for ev in mouse_motion.read() {
        total_mouse_delta.x += ev.delta.x;
        total_mouse_delta.y += ev.delta.y;
    }

    // Collect mouse wheel delta
    let mut total_wheel_delta = 0.0;
    for ev in mouse_wheel.read() {
        total_wheel_delta += ev.y;
    }

    // Prepare input for business logic
    let input = CameraInput {
        move_forward: keyboard_input.pressed(KeyCode::KeyW),
        move_backward: keyboard_input.pressed(KeyCode::KeyS),
        move_left: keyboard_input.pressed(KeyCode::KeyA),
        move_right: keyboard_input.pressed(KeyCode::KeyD),
        sprint: keyboard_input.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
        mouse_right_pressed: mouse_input.pressed(MouseButton::Right),
        mouse_delta: total_mouse_delta,
        mouse_wheel_delta: total_wheel_delta,
    };

    // Call business logic
    let update = calculate_camera_transform(
        transform.translation,
        transform.rotation,
        &input,
        dt,
    );

    // Apply results to ECS
    transform.translation = update.translation;
    transform.rotation = update.rotation;
}
