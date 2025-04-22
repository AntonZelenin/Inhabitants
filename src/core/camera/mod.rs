pub(crate) mod components;

use crate::core::camera::components::{MainCamera, MainCameraTarget};
use crate::core::state::GameState;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use std::f32::consts::PI;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainCamera>()
            .register_type::<MainCameraTarget>()
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (zoom_control.run_if(in_state(GameState::InGame)),))
            .add_systems(
                PostUpdate,
                (camera_control.run_if(in_state(GameState::InGame)),),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(4.0, 4.0, 12.0).looking_at(Vec3::new(0.0, 0.0, 0.5), Vec3::Y),
        MainCamera,
    ));

    commands.spawn((
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        DirectionalLight::default(),
    ));

    info!("Camera spawned");
}

fn zoom_control(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut camera_projection_q: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = camera_projection_q.single_mut();

    for mouse_wheel_event in mouse_wheel_events.read() {
        if mouse_wheel_event.y < 0.0 {
            projection.scale *= 1.1;
        } else if mouse_wheel_event.y > 0.0 {
            projection.scale *= 0.9;
        }
    }

    projection.scale = projection.scale.clamp(0.8, 4.5);
}

fn camera_control(
    mut camera_transforms: Query<&mut Transform, (With<MainCamera>, Without<MainCameraTarget>)>,
    player_transforms: Query<&Transform, With<MainCameraTarget>>,
) {
    if let Ok(player_transform) = player_transforms.get_single() {
        let mut camera_transform = camera_transforms.single_mut();

        let lerp = camera_transform
            .translation
            .lerp(player_transform.translation, 0.1);
        camera_transform.translation.x = lerp.x;
        camera_transform.translation.y = lerp.y;
    }
}
