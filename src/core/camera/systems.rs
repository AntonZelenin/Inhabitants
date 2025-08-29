use crate::core::camera::components::{MainCamera, MainCameraTarget};
use crate::planet::components::CameraLerp;
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::log::info;
use bevy::math::{EulerRot, Quat, Vec3};
use bevy::pbr::DirectionalLight;
use bevy::prelude::{
    Camera3d, Commands, EventReader, KeyCode, MouseButton, Query, Res, Time, Transform, With,
    Without,
};
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