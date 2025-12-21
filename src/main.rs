// I made a little flight sim here
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
    pub default_position: Vec3,
    pub default_lookat: Vec3,
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
            default_position: Vec3 {
                x: 0.0,
                y: 5.0,
                z: -18.0,
            },
            default_lookat: Vec3 {
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
    x: f32,
    y: f32,
    z: f32,
} // all three spacial axis

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
        .insert_resource(RotationOfSubject(quat(0.0, 0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (input_system, subject_movement, print_fps, camera_movement),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_settings: Res<CameraSettings>,
) {
    // circular base
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("landscapeFS.glb")),
    ));
    // cube
    commands
        .spawn(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("simplPlane.glb")),
        ))
        .insert(Aircraft)
        .with_children(|parent| {
            parent
                .spawn((
                    Camera3d::default(),
                    Atmosphere::EARTH,
                    Exposure::SUNLIGHT,
                    Tonemapping::AgX,
                    Bloom::NATURAL,
                    Transform::from_translation(camera_settings.default_position)
                        .looking_at(camera_settings.default_lookat, Vec3::Y),
                ))
                .insert(FollowCamera);
        });

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,

            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn camera_movement(
    mut camera: Single<&mut Transform, With<Camera>>,
    camera_settings: Res<CameraSettings>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
) {
    let delta = mouse_motion.delta;
    // Mouse motion is one of the few inputs that should not be multiplied by delta time,
    // as we are already receiving the full movement since the last frame was rendered. Multiplying
    // by delta time here would make the movement slower that it should be.
    let delta_pitch = delta.y * camera_settings.pitch_speed;
    let delta_yaw = delta.x * camera_settings.yaw_speed;

    // Obtain the existing pitch, yaw, and roll values from the transform.
    let (yaw, pitch, roll) = camera.rotation.to_euler(EulerRot::YXZ);

    // Establish the new yaw and pitch, preventing the pitch value from exceeding our limits.
    let pitch = (pitch + delta_pitch).clamp(
        camera_settings.pitch_range.start,
        camera_settings.pitch_range.end,
    );

    let yaw = yaw + delta_yaw;

    // Adjust the translation to maintain the correct orientation toward the orbit target.
    // In our example it's a static target, but this could easily be customized.
    let target = camera_settings.default_lookat;
    if mouse_buttons.pressed(MouseButton::Right) {
        camera.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        camera.translation = target - camera.forward() * camera_settings.orbit_distance;
    }

    // camera reset logic
    if mouse_buttons.just_pressed(MouseButton::Left) {
        camera.translation = camera_settings.default_position;
        camera.look_at(target, Vec3::Y);
    }
}

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps = fps.unwrap().value().unwrap_or(0.0);
    #[cfg(debug_assertions)]
    info!("{:?}", fps);
}

fn subject_movement(
    mut query: Query<&mut Transform, With<Aircraft>>,
    time: Res<Time>,
    input: Res<InputAxis>,
    mut rotation: ResMut<RotationOfSubject>,
) {
    let delta = time.delta_secs();
    for mut transform in &mut query {
        let rotation_x = Quat::from_rotation_x(input.y * delta);
        let rotation_z = Quat::from_rotation_z(input.x * delta);
        transform.rotate_local(rotation_x);
        transform.rotate_local(rotation_z);

        let forward = transform.back();
        transform.translation += forward * delta * 10.; //* input.z; // why do my rightstick only work on web??

        rotation.0 = transform.rotation;
    }
}

fn input_system(
    mut is_gamepad_connected: ResMut<IsGamepadConnected>,
    mut input: ResMut<InputAxis>,
    gamepads: Query<(Entity, &Gamepad)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
    gamepad_settings: Res<GamepadSettings>,
) {
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
        if connection_event.connected() == true {
            is_gamepad_connected.0 = true;
        }
    }

    // switch to gamepad when connected
    if is_gamepad_connected.0 == false {
        button_input(input, keyboard_input);
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

fn button_input(mut input: ResMut<'_, InputAxis>, keyboard_input: Res<'_, ButtonInput<KeyCode>>) {
    // Z axis (forward)
    if keyboard_input.pressed(KeyCode::KeyW) {
        input.y = 1.0;
    } else if keyboard_input.pressed(KeyCode::KeyS) {
        input.y = -1.0;
    } else {
        input.y = 0.0
    }

    // X axis (left/right)
    if keyboard_input.pressed(KeyCode::KeyA) {
        input.x = -1.0
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        input.x = 1.0;
    } else {
        input.x = 0.0
    }

    // Y axis (up/down)
    if keyboard_input.pressed(KeyCode::KeyQ) {
        input.z = 1.0
    } else if keyboard_input.pressed(KeyCode::KeyE) {
        input.z = -1.0;
    } else {
        input.z = 0.0
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
        let right_stick_y = gamepad.get(GamepadAxis::RightStickX).unwrap();

        // xy and something else
        return (left_stick_x, left_stick_y, right_stick_y);
    }

    // return zero if nothing is connected, but this technially shouldn't happen
    return (0.0, 0.0, 0.0);
}
