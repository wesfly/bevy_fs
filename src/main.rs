// I made a little flight sim here

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::gamepad::GamepadConnectionEvent,
    light::light_consts::lux,
    pbr::Atmosphere,
    post_process::bloom::Bloom,
    prelude::*,
};

const TOGGLE_CONTROL_SNAPPING: bool = true;
const CONTROL_SNAPPING_THRESHOLD: f32 = 0.075;

#[derive(Resource)]
struct RotationOfSubject(Quat);

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct Aircraft;

#[derive(Resource)]
struct Input {
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
        .insert_resource(Input {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        })
        .insert_resource(IsGamepadConnected(false))
        .insert_resource(RotationOfSubject(quat(0.0, 0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .add_systems(Update, (input_system, subject_movement, print_fps))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                    Transform::from_xyz(0.0, 9.0, -18.0).looking_at(
                        Vec3 {
                            x: 0.0,
                            y: 1.0,
                            z: 0.0,
                        },
                        Vec3::Y,
                    ),
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

fn print_fps(diagnostics: Res<DiagnosticsStore>) {
    let fps = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS);
    let fps = fps.unwrap().value().unwrap_or(0.0);
    info!("{:?}", fps);
}

fn subject_movement(
    mut query: Query<&mut Transform, With<Aircraft>>,
    time: Res<Time>,
    input: Res<Input>,
    mut rotation: ResMut<RotationOfSubject>,
) {
    let delta = time.delta_secs();
    for mut transform in &mut query {
        let rotation_x = Quat::from_rotation_x(input.y * delta);
        let rotation_z = Quat::from_rotation_z(input.x * delta);
        transform.rotate_local(rotation_x);
        transform.rotate_local(rotation_z);

        let forward = transform.back();
        transform.translation += forward * delta * 10.0;

        rotation.0 = transform.rotation;
    }
}

fn input_system(
    mut is_gamepad_connected: ResMut<IsGamepadConnected>,
    mut input: ResMut<Input>,
    gamepads: Query<(Entity, &Gamepad)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
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

        // Control values snap to zero when under a certain value
        if TOGGLE_CONTROL_SNAPPING {
            if -CONTROL_SNAPPING_THRESHOLD < gamepad_input.0
                && gamepad_input.0 < CONTROL_SNAPPING_THRESHOLD
            {
                input.x = 0.0
            }
            if -CONTROL_SNAPPING_THRESHOLD < gamepad_input.1
                && gamepad_input.1 < CONTROL_SNAPPING_THRESHOLD
            {
                input.y = 0.0
            }
            if -CONTROL_SNAPPING_THRESHOLD < gamepad_input.2
                && gamepad_input.2 < CONTROL_SNAPPING_THRESHOLD
            {
                input.z = 0.0
            }
        }
    }
}

fn button_input(mut input: ResMut<'_, Input>, keyboard_input: Res<'_, ButtonInput<KeyCode>>) {
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

        let right_stick_y = 0.0;

        // xy and something else
        return (left_stick_x, left_stick_y, right_stick_y);
    }

    // return zero if nothing is connected, but this technially shouldn't happen
    return (0.0, 0.0, 0.0);
}
