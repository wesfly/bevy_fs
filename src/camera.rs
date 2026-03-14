use crate::{
    aircraft::{Aircraft, AircraftState},
    input::Keymap,
};
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

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let cockpit_pos = Vec3 {
            x: 0.0,
            y: 1.2,
            z: -3.27,
        };
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 25.0,
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
                y: 2.5,
                z: 0.0,
            },
            cockpit_default_position: cockpit_pos,
            tail_default_position: Vec3 {
                x: 0.0,
                y: 4.0,
                z: 7.0,
            },
            view: CameraView::Follow,
        }
    }
}

#[derive(Resource)]
pub struct CameraPosition(pub Transform);

impl Camera {
    pub fn controller(
        mut camera: Single<&mut Transform, (With<Camera>, Without<Aircraft>)>,
        mut camera_settings: ResMut<CameraSettings>,
        state: Res<AircraftState>,
        aircraft: Single<&Transform, With<Aircraft>>,
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        mouse_motion: Res<AccumulatedMouseMotion>,
        keyboard_input: Res<'_, ButtonInput<KeyCode>>,
        keymap: Res<Keymap>,
        mut projection: Single<&mut Projection, With<Camera>>,
        mut scroll_events: MessageReader<MouseWheel>,
        mut camera_pos: ResMut<CameraPosition>,
    ) {
        camera_settings.cockpit_default_position = match state.aircraft_type {
            crate::aircraft::AircraftTypes::Helicopter => Vec3 {
                x: 0.38,
                y: 1.2,
                z: -2.6,
            },
            crate::aircraft::AircraftTypes::Aeroplane => Vec3 {
                x: 0.0,
                y: 1.2,
                z: -3.27,
            },
        };
        let delta = mouse_motion.delta;

        let delta_pitch = -delta.y * camera_settings.pitch_speed;
        let delta_yaw = -delta.x * camera_settings.yaw_speed;

        let (yaw, pitch, roll) = camera_pos.0.rotation.to_euler(EulerRot::YXZ);

        let (mut pitch, mut yaw) = match mouse_buttons.pressed(MouseButton::Right) {
            true => (
                (pitch + delta_pitch).clamp(
                    camera_settings.pitch_range.start,
                    camera_settings.pitch_range.end,
                ),
                (yaw + delta_yaw),
            ),
            false => (pitch, yaw),
        };

        match camera_settings.view {
            CameraView::Cockpit => {
                camera.translation = aircraft.translation
                    + aircraft.rotation * camera_settings.cockpit_default_position;
                camera.rotation = aircraft.rotation * camera_pos.0.rotation;
            }
            CameraView::Follow => {
                let target = camera_settings.follow_default_lookat + aircraft.translation;

                let forward = aircraft.rotation * Vec3::Z;
                let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();

                // Ignoring roll to create a MSFS-like camera
                let no_roll = Quat::from_euler(
                    EulerRot::YXZ,
                    flat_forward.x.atan2(flat_forward.z), // yaw
                    0.0, //-forward.y.asin(),                    // pitch
                    0.0, // roll
                );
                camera.rotation = no_roll * camera_pos.0.rotation;

                camera.translation = target - camera.forward() * camera_settings.orbit_distance;

                if keyboard_input.just_pressed(keymap.reset_camera) {
                    camera.translation = aircraft.translation
                        + aircraft.rotation * camera_settings.follow_default_position;
                    pitch = 0.0;
                    yaw = 0.0;
                }
            }
            CameraView::Tail => {
                camera.translation = aircraft.translation
                    + aircraft.rotation * camera_settings.tail_default_position;
                camera.rotation = aircraft.rotation * camera_pos.0.rotation;
            }
        }

        camera_pos.0.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);

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
}
