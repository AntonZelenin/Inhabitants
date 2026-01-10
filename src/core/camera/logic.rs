use bevy::math::{Quat, Vec3};

pub struct CameraInput {
    pub move_forward: bool,
    pub move_backward: bool,
    pub move_left: bool,
    pub move_right: bool,
    pub sprint: bool,
    pub mouse_right_pressed: bool,
    pub mouse_delta: Vec3,
    pub mouse_wheel_delta: f32,
}

pub struct CameraTransformUpdate {
    pub translation: Vec3,
    pub rotation: Quat,
}

/// Calculate camera movement and rotation based on input
/// Returns the new transform values
pub fn calculate_camera_transform(
    current_translation: Vec3,
    current_rotation: Quat,
    input: &CameraInput,
    delta_time: f32,
) -> CameraTransformUpdate {
    let mut translation = current_translation;
    let mut rotation = current_rotation;

    // Calculate movement speed
    let mut speed = 5.0;
    if input.sprint {
        speed *= 5.0;
    }

    // Calculate direction vectors based on current rotation
    let forward = rotation.mul_vec3(Vec3::new(0.0, 0.0, -1.0));
    let right = rotation.mul_vec3(Vec3::new(1.0, 0.0, 0.0));
    let mut dir = Vec3::ZERO;

    // Apply movement input
    if input.move_forward {
        dir += forward;
    }
    if input.move_backward {
        dir -= forward;
    }
    if input.move_left {
        dir -= right;
    }
    if input.move_right {
        dir += right;
    }
    if dir.length_squared() > 0.0 {
        translation += dir.normalize() * speed * delta_time;
    }

    // Apply mouse rotation
    if input.mouse_right_pressed && input.mouse_delta.length_squared() > 0.0 {
        let yaw = Quat::from_rotation_y(-input.mouse_delta.x * 0.002);
        let pitch = Quat::from_rotation_x(-input.mouse_delta.y * 0.002);
        rotation = yaw * rotation * pitch;
    }

    // Apply mouse wheel movement
    if input.mouse_wheel_delta.abs() > 0.0 {
        translation += forward * input.mouse_wheel_delta * 0.5;
    }

    CameraTransformUpdate {
        translation,
        rotation,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::f32::consts::PI;

    fn default_input() -> CameraInput {
        CameraInput {
            move_forward: false,
            move_backward: false,
            move_left: false,
            move_right: false,
            sprint: false,
            mouse_right_pressed: false,
            mouse_delta: Vec3::ZERO,
            mouse_wheel_delta: 0.0,
        }
    }

    #[test]
    fn test_no_input_no_change() {
        let start_pos = Vec3::new(1.0, 2.0, 3.0);
        let start_rot = Quat::IDENTITY;
        let input = default_input();

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        assert_eq!(result.translation, start_pos);
        assert_eq!(result.rotation, start_rot);
    }

    #[rstest]
    #[case(true, false, false, false, 0.0, 0.0, -5.0)] // forward
    #[case(false, true, false, false, 0.0, 0.0, 5.0)]  // backward
    #[case(false, false, true, false, -5.0, 0.0, 0.0)] // left
    #[case(false, false, false, true, 5.0, 0.0, 0.0)]  // right
    fn test_basic_movement(
        #[case] move_forward: bool,
        #[case] move_backward: bool,
        #[case] move_left: bool,
        #[case] move_right: bool,
        #[case] expected_x: f32,
        #[case] expected_y: f32,
        #[case] expected_z: f32,
    ) {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.move_forward = move_forward;
        input.move_backward = move_backward;
        input.move_left = move_left;
        input.move_right = move_right;

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        assert!((result.translation.x - expected_x).abs() < 0.01, "x was {}, expected {}", result.translation.x, expected_x);
        assert!((result.translation.y - expected_y).abs() < 0.01, "y was {}, expected {}", result.translation.y, expected_y);
        assert!((result.translation.z - expected_z).abs() < 0.01, "z was {}, expected {}", result.translation.z, expected_z);
    }

    #[rstest]
    #[case(false, -5.0)]  // Normal speed: 5.0
    #[case(true, -25.0)]  // Sprint speed: 5.0 * 5.0 = 25.0
    fn test_sprint_multiplier(#[case] sprint: bool, #[case] expected_z: f32) {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.move_forward = true;
        input.sprint = sprint;

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        assert!((result.translation.z - expected_z).abs() < 0.01);
    }

    #[test]
    fn test_diagonal_movement_normalized() {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.move_forward = true;
        input.move_right = true;

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        // Should move diagonally but normalized (total distance = 5.0)
        let distance = result.translation.length();
        assert!((distance - 5.0).abs() < 0.01);
    }

    #[rstest]
    #[case(2.0, -1.0)]    // Scroll forward: 2.0 * 0.5 = 1.0 in -Z direction
    #[case(-2.0, 1.0)]    // Scroll backward: -2.0 * 0.5 = -1.0 in -Z direction
    #[case(4.0, -2.0)]    // Larger scroll forward
    #[case(0.0, 0.0)]     // No scroll
    fn test_mouse_wheel_movement(#[case] wheel_delta: f32, #[case] expected_z: f32) {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.mouse_wheel_delta = wheel_delta;

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        assert!((result.translation.z - expected_z).abs() < 0.01);
    }

    #[rstest]
    #[case(true, 100.0, 0.0)]    // Yaw with right button pressed
    #[case(true, 0.0, 100.0)]    // Pitch with right button pressed
    #[case(true, 50.0, 50.0)]    // Both yaw and pitch
    fn test_mouse_rotation(
        #[case] mouse_right_pressed: bool,
        #[case] delta_x: f32,
        #[case] delta_y: f32,
    ) {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.mouse_right_pressed = mouse_right_pressed;
        input.mouse_delta = Vec3::new(delta_x, delta_y, 0.0);

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        // Rotation should have changed
        assert_ne!(result.rotation, start_rot);
        // Position should not change
        assert_eq!(result.translation, start_pos);
    }


    #[test]
    fn test_mouse_rotation_requires_right_button() {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.mouse_right_pressed = false;
        input.mouse_delta = Vec3::new(100.0, 100.0, 0.0);

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        // Rotation should NOT change without right button
        assert_eq!(result.rotation, start_rot);
    }

    #[rstest]
    #[case(0.5, -2.5)]   // Half delta time: 5.0 * 0.5 = 2.5
    #[case(1.0, -5.0)]   // Normal delta time: 5.0 * 1.0 = 5.0
    #[case(2.0, -10.0)]  // Double delta time: 5.0 * 2.0 = 10.0
    #[case(0.1, -0.5)]   // Small delta time: 5.0 * 0.1 = 0.5
    fn test_delta_time_affects_movement(#[case] delta_time: f32, #[case] expected_z: f32) {
        let start_pos = Vec3::ZERO;
        let start_rot = Quat::IDENTITY;
        let mut input = default_input();
        input.move_forward = true;

        let result = calculate_camera_transform(start_pos, start_rot, &input, delta_time);

        assert!((result.translation.z - expected_z).abs() < 0.01);
    }

    #[test]
    fn test_rotated_camera_forward_movement() {
        let start_pos = Vec3::ZERO;
        // Rotate 90 degrees around Y axis (now facing -X)
        let start_rot = Quat::from_rotation_y(PI / 2.0);
        let mut input = default_input();
        input.move_forward = true;

        let result = calculate_camera_transform(start_pos, start_rot, &input, 1.0);

        // Should move in the direction the camera is facing (-X)
        assert!((result.translation.x - (-5.0)).abs() < 0.01, "x was {}", result.translation.x);
        assert!((result.translation.y).abs() < 0.01);
        assert!((result.translation.z).abs() < 0.01);
    }
}
