// I made a little flight simulator here. Check out the README for further information.
// If you have fixes or want to contribute, just make a pull request (unless it's AI-generated)

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::{gamepad::GamepadConnectionEvent, mouse::AccumulatedMouseMotion},
    light::light_consts::lux,
    pbr::Atmosphere,
    post_process::bloom::Bloom,
    prelude::*,
};
use std::{f32::consts::FRAC_PI_2, ops::Range};

mod aircraft_mechanics;

#[derive(Resource)]
struct Keymap {
    reset_camera: KeyCode,
    up: KeyCode,
    down: KeyCode,
    rudder_left: KeyCode,
    rudder_right: KeyCode,
    roll_left: KeyCode,
    roll_right: KeyCode,
}
impl Default for Keymap {
    fn default() -> Self {
        Self {
            reset_camera: KeyCode::KeyR,
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            rudder_left: KeyCode::KeyA,
            rudder_right: KeyCode::KeyD,
            roll_left: KeyCode::KeyQ,
            roll_right: KeyCode::KeyE,
        }
    }
}

#[derive(Resource)]
struct GamepadSettings {
    control_snapping_enabled: bool,
    control_snapping_treshold: f32,
}

impl Default for GamepadSettings {
    fn default() -> Self {
        Self {
            control_snapping_enabled: true,
            control_snapping_treshold: 0.075,
        }
    }
}

#[derive(Debug, Resource)]
struct CameraSettings {
    pub orbit_distance: f32,
    pub pitch_speed: f32,
    // Clamp pitch to this range
    pub pitch_range: Range<f32>,
    pub yaw_speed: f32,
    pub follow_default_position: Vec3,
    pub follow_default_lookat: Vec3,
}

impl Default for CameraSettings {
    fn default() -> Self {
        // Limiting pitch stops some unexpected rotation past 90° up or down.
        let pitch_limit = FRAC_PI_2 - 0.01;
        Self {
            orbit_distance: 40.0,
            pitch_speed: 0.003,
            pitch_range: -pitch_limit..pitch_limit,
            yaw_speed: 0.004,
            follow_default_position: Vec3 {
                x: 0.0,
                y: 10.0,
                z: -30.0,
            },
            follow_default_lookat: Vec3 {
                x: 0.0,
                y: 2.0,
                z: 0.0,
            },
        }
    }
}

#[derive(Resource)]
struct RotationOfSubject(Quat);

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct Aircraft;

#[derive(Resource)]
struct InputAxis {
    x: f32, // Pitch
    y: f32, // Yaw
    z: f32, // Roll
}

#[derive(Resource)]
struct IsGamepadConnected(bool);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(InputAxis {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
        .insert_resource(GamepadSettings::default())
        .insert_resource(IsGamepadConnected(false))
        .insert_resource(CameraSettings::default())
        .insert_resource(Keymap::default())
        .insert_resource(RotationOfSubject(quat(0.0, 0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input_system,
                aircraft_mechanics::aircraft_mechanics,
                print_fps,
                camera_movement,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_settings: Res<CameraSettings>,
) {
    // landscape
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("landscapeFS.glb")),
    ));

    // aircraft
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            SceneRoot(asset_server.load("aircraft.glb#Scene0")),
            Aircraft,
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_translation(camera_settings.follow_default_position)
                    .looking_at(camera_settings.follow_default_lookat, Vec3::Y),
                Atmosphere::EARTH,
                Exposure::SUNLIGHT,
                Tonemapping::AgX,
                Bloom::NATURAL,
                FollowCamera,
            ));
        });

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps = fps.unwrap().value().unwrap_or(0.0);
    #[cfg(debug_assertions)]
    info!("{:?}", fps);
}

fn camera_movement(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    keymap: Res<Keymap>,
) {
    let delta = mouse_motion.delta;
    let delta_pitch = delta.y * camera_settings.pitch_speed;
    let delta_yaw = delta.x * camera_settings.yaw_speed;

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );

    let yaw = yaw + delta_yaw;

    // Adjust the translation to maintain the correct orientation toward the orbit target.
    // In our example it's a static target, but this could easily be customized.
    let target = camera_settings.follow_default_lookat;
    if mouse_buttons.pressed(MouseButton::Right) {
        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        camera.translation = target - camera.forward() * camera_settings.orbit_distance;
    }

    // camera reset logic
    if keyboard_input.just_pressed(keymap.reset_camera) {
        camera.translation = camera_settings.follow_default_position;
        camera.look_at(target, Vec3::Y);
    }
}

fn input_system(
    mut is_gamepad_connected: ResMut<IsGamepadConnected>,
    mut input: ResMut<InputAxis>,
    gamepads: Query<(Entity, &Gamepad)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
    gamepad_settings: Res<GamepadSettings>,
    keymap: Res<Keymap>,
) {
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
        if connection_event.connected() == true {
            is_gamepad_connected.0 = true;
        }
    }

    // Switch to gamepad when connected
    if is_gamepad_connected.0 == false {
        button_input_system(input, keyboard_input, keymap);
    } else if is_gamepad_connected.0 == true {
        let gamepad_input = gamepad_input_system(gamepads, connection_events);

        input.x = gamepad_input.0;
        input.y = gamepad_input.1;
        input.z = gamepad_input.2;

        let threshold = gamepad_settings.control_snapping_treshold;
        let threshold_range = -threshold..threshold;

        // Control values snap to zero when in a certain range
        if gamepad_settings.control_snapping_enabled {
            if threshold_range.contains(&gamepad_input.0) {
                input.x = 0.0
            }
            if threshold_range.contains(&gamepad_input.1) {
                input.y = 0.0
            }
            if threshold_range.contains(&gamepad_input.2) {
                input.z = 0.0
            }
        }
    }
}

fn button_input_system(
    mut input: ResMut<'_, InputAxis>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    keymap: Res<'_, Keymap>,
) {
    // X axis (pitch up/down)
    if keyboard_input.pressed(keymap.up) {
        input.x = 1.0;
    } else if keyboard_input.pressed(keymap.down) {
        input.x = -1.0;
    } else {
        input.x = 0.0
    }

    // Z axis (yaw left/right)
    if keyboard_input.pressed(keymap.rudder_left) {
        input.z = -1.0
    } else if keyboard_input.pressed(keymap.rudder_right) {
        input.z = 1.0;
    } else {
        input.z = 0.0
    }

    // Y axis (roll left/right)
    if keyboard_input.pressed(keymap.roll_left) {
        input.y = 1.0
    } else if keyboard_input.pressed(keymap.roll_right) {
        input.y = -1.0;
    } else {
        input.y = 0.0
    }
}

fn gamepad_input_system(
    gamepads: Query<(Entity, &Gamepad)>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
) -> (f32, f32, f32) {
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
    }
    for (_entity, gamepad) in &gamepads {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap();
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap();

        // Should just use the first gamepad that is connected, having two is rare
        return (left_stick_y, right_stick_x, left_stick_x);
    }

    // return zero if nothing is connected, but this technially shouldn't happen
    return (0.0, 0.0, 0.0);
}
