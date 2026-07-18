use crate::{
    aircraft::{Aircraft, AircraftState},
    bevy_to_aerospace_coords,
    input::Keymap,
};
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{Exposure, Hdr},
    camera_controller::free_camera::FreeCamera,
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel},
    light::AtmosphereEnvironmentMapLight,
    pbr::{AtmosphereSettings, ScreenSpaceReflections},
    post_process::{bloom::Bloom, motion_blur::MotionBlur},
    prelude::*,
};
use big_space::prelude::*;
use std::{
    f32::consts::{FRAC_PI_2, PI},
    ops::Range,
};

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
    tail_default_position: Vec3,
    pub view: CameraView,
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 25.0,
            pitch_speed: 0.001,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.001,
            follow_default_position: Vec3 {
                x: -20.0,
                y: 4.0,
                z: 0.0,
            },
            follow_default_lookat: Vec3 {
                x: 0.0,
                y: 2.5,
                z: 0.0,
            },
            tail_default_position: Vec3 {
                x: 0.0,
                y: 4.0,
                z: 7.0,
            },
            view: CameraView::Follow,
        }
    }
}

const CAM_EXPOSURE: Exposure = Exposure { ev100: 13.0 };

#[derive(Resource)]
pub struct CameraPosition(pub Transform);

impl Camera {
    pub fn spawn(cell_coord: CellCoord, parent: Entity) -> impl Bundle {
        (
            FloatingOrigin,
            cell_coord,
            ChildOf(parent),
            Camera3d::default(),
            AtmosphereSettings {
                rendering_method: bevy::pbr::AtmosphereMode::Raymarched,
                ..default()
            },
            AtmosphereEnvironmentMapLight::default(),
            CAM_EXPOSURE,
            Tonemapping::AcesFitted,
            Bloom::NATURAL,
            Projection::from(PerspectiveProjection {
                fov: 50.0_f32.to_radians(),
                ..default()
            }),
            FreeCamera::default(),
            (
                Msaa::Off,
                TemporalAntiAliasing::default(),
                ScreenSpaceReflections {
                    min_perceptual_roughness: 0.0..0.0,
                    ..default()
                },
                Hdr,
                MotionBlur {
                    shutter_angle: 0.5,
                    samples: 2,
                },
            ),
            // crate::camera::Camera,
        )
    }

    pub fn controller(
        mut camera: Single<&mut Transform, (With<Camera>, Without<Aircraft>)>,
        camera_settings: Res<CameraSettings>,
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
        let cockpit_default_position = match state.aircraft_type {
            crate::aircraft::AircraftTypes::Helicopter => Vec3 {
                x: 0.38,
                y: 1.2,
                z: -2.6,
            },
            crate::aircraft::AircraftTypes::J3Cub => Vec3 {
                x: 0.2,
                y: 0.0,
                z: -0.5,
            },
            _ => Vec3 {
                x: 2.8,
                y: 0.0,
                z: -1.2,
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
                camera.translation =
                    aircraft.translation + aircraft.rotation * cockpit_default_position;
                camera.rotation = match state.aircraft_type {
                    crate::aircraft::AircraftTypes::Helicopter => {
                        aircraft.rotation * camera_pos.0.rotation
                    }
                    _ => aircraft.rotation * bevy_to_aerospace_coords() * camera_pos.0.rotation,
                };
            }
            CameraView::Follow => {
                let target = camera_settings.follow_default_lookat + aircraft.translation;

                let forward = match state.aircraft_type {
                    crate::aircraft::AircraftTypes::Helicopter => aircraft.local_z(),
                    _ => -aircraft.local_x(),
                };
                let flat_forward = Vec3::new(forward.x, 0.0, forward.z).normalize();

                // Ignoring roll to create a MSFS-like camera
                let no_roll = Quat::from_euler(
                    EulerRot::YXZ,
                    flat_forward.x.atan2(flat_forward.z), // yaw
                    0.0,                                  // pitch (-forward.y.asin(),)
                    0.0,                                  // roll
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
                camera.translation = match state.aircraft_type {
                    crate::aircraft::AircraftTypes::Helicopter => {
                        aircraft.translation
                            + aircraft.rotation * camera_settings.tail_default_position
                    }
                    _ => {
                        aircraft.translation
                            + aircraft.rotation
                                * bevy_to_aerospace_coords()
                                * camera_settings.tail_default_position
                    }
                };

                camera.rotation = match state.aircraft_type {
                    crate::aircraft::AircraftTypes::Helicopter => {
                        aircraft.rotation * camera_pos.0.rotation
                    }
                    _ => aircraft.rotation * bevy_to_aerospace_coords() * camera_pos.0.rotation,
                };
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

// https://github.com/evroon/bevy-open-world/blob/master/crates/bevy-terrain/src/camera.rs
pub fn rotate_sun(
    mut suns: Query<&mut Transform, With<DirectionalLight>>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let mut sun_vert_rot_factor = 0.0;
    let mut sun_hor_rot_factor = 0.0;

    if keys.pressed(KeyCode::KeyH) {
        sun_vert_rot_factor -= 0.1;
    }
    if keys.pressed(KeyCode::KeyJ) {
        sun_vert_rot_factor += 0.1;
    }
    if keys.pressed(KeyCode::KeyK) {
        sun_hor_rot_factor -= 0.2;
    }
    if keys.pressed(KeyCode::KeyL) {
        sun_hor_rot_factor += 0.2;
    }

    suns.iter_mut().for_each(|mut tf| {
        tf.rotate_x(time.delta_secs() * PI * sun_vert_rot_factor);
        tf.rotate_y(time.delta_secs() * PI * sun_hor_rot_factor)
    });
}
