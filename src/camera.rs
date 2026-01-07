use crate::{CameraSettings, FollowCamera, input::Keymap};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};

pub fn camera_controller(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    keymap: Res<Keymap>,
    mut projection: Single<&mut Projection, With<FollowCamera>>,
    mut scroll_events: MessageReader<MouseWheel>,
) {
    let cockpit_cam: bool = camera_settings.view == 1;

    let delta = mouse_motion.delta;
    let delta_pitch;
    let delta_yaw;

    if cockpit_cam {
        delta_pitch = -delta.y * camera_settings.pitch_speed;
        delta_yaw = -delta.x * camera_settings.yaw_speed;
    } else {
        delta_pitch = delta.y * camera_settings.pitch_speed;
        delta_yaw = delta.x * camera_settings.yaw_speed;
    }

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );

    let yaw = yaw + delta_yaw;

    if cockpit_cam {
        if mouse_buttons.pressed(MouseButton::Right) {
            camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
        camera.translation = camera_settings.cockpit_default_position;
    } else {
        let target = camera_settings.follow_default_lookat;
        if mouse_buttons.pressed(MouseButton::Right) {
            camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
        camera.translation = target - camera.forward() * camera_settings.orbit_distance;

        if keyboard_input.just_pressed(keymap.reset_camera) {
            camera.translation = camera_settings.follow_default_position;
            camera.look_at(target, Vec3::Y);
        }
    }

    let Projection::Perspective(perspective) = projection.as_mut() else {
        return;
    };

    for event in scroll_events.read() {
        match event.unit {
            MouseScrollUnit::Line => perspective.fov -= event.y * 0.05,
            MouseScrollUnit::Pixel => {}
        }
    }

    perspective.fov = perspective.fov.clamp(0.1, std::f32::consts::FRAC_PI_2);
}
