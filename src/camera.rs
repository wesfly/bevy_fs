use crate::{FollowCamera, input::Keymap};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use std::{f32::consts::FRAC_PI_2, ops::Range};

#[derive(Debug, Resource)]
pub struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    // Clamp pitch to this range
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
    pub follow_default_position: Vec3,
    pub follow_default_lookat: Vec3,
    pub cockpit_default_position: Vec3,
    pub view: u8,
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 20.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.004,
            follow_default_position: Vec3 {
                x: 0.0,
                y: 4.0,
                z: 20.0,
            },
            follow_default_lookat: Vec3 {
                x: 0.0,
                y: 0.5,
                z: 0.0,
            },
            cockpit_default_position: Vec3 {
                x: 0.4,
                y: 0.8,
                z: -3.0,
            },
            view: 0,
        }
    }
}

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
