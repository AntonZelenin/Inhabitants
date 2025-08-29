pub(crate) mod components;
mod systems;

use crate::core::camera::components::*;
use crate::core::camera::systems::*;
use crate::core::state::GameState;
use bevy::prelude::*;

pub(crate) struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MainCamera>()
            .register_type::<MainCameraTarget>()
            .add_systems(Startup, spawn_camera)
            .add_systems(
                PostUpdate,
                camera_control.run_if(in_state(GameState::InGame)),
            );
    }
}
