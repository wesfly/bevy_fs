use crate::{Settings, aircraft::Aircraft, input::Keymap};
use bevy::{
    input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel},
    prelude::*,
};
use std::{f32::consts::FRAC_PI_2, ops::Range};

#[derive(Component)]
pub struct Camera;

#[derive(Debug)]
pub enum CameraView {
    Cockpit,
    Follow,
    Tail,
}

#[derive(Debug, Resource)]
pub struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
    follow_default_position: Vec3,
    follow_default_lookat: Vec3,
    cockpit_default_position: Vec3,
    tail_default_position: Vec3,
    pub view: CameraView,
}

impl CameraSettings {
    pub fn init(settings: &Settings) -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let cockpit_pos = match settings.aircraft {
            crate::aircraft::AircraftTypes::Aeroplane => Vec3 {
                x: 0.0,
                y: 1.2,
                z: -3.27,
            },
            crate::aircraft::AircraftTypes::Helicopter => Vec3 {
                x: 0.38,
                y: 1.2,
                z: -2.6,
            },
        };
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 20.0,
            pitch_speed: 0.001,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.001,
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
            cockpit_default_position: cockpit_pos,
            tail_default_position: Vec3 {
                x: 0.5,
                y: 3.0,
                z: 8.0,
            },
            view: CameraView::Follow,
        }
    }
}

pub fn camera_controller(
    mut camera: Single<&mut Transform, (With<Camera>, Without<Aircraft>)>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    keymap: Res<Keymap>,
    mut projection: Single<&mut Projection, With<Camera>>,
    mut scroll_events: MessageReader<MouseWheel>,
) {
    let delta = mouse_motion.delta;

    let delta_pitch = -delta.y * camera_settings.pitch_speed;
    let delta_yaw = -delta.x * camera_settings.yaw_speed;

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );

    let yaw = yaw + delta_yaw;

    match camera_settings.view {
        CameraView::Cockpit => {
            if mouse_buttons.pressed(MouseButton::Right) {
                camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
            }
            camera.translation = camera_settings.cockpit_default_position;
        }
        CameraView::Tail => {
            if mouse_buttons.pressed(MouseButton::Right) {
                camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
            }
            camera.translation = camera_settings.tail_default_position;
        }
        CameraView::Follow => {
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
