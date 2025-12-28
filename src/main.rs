/*
I made a little flight simulator here. Check out the README for further information.
If you have fixes or want to contribute, just make a pull request (unless it's AI-generated)

I don't exactly know where these warnings are coming from (they're probably from the aircraft and its hitbox), I'm just
ignoring them.
*/

pub const ENABLE_GAMEPAD: bool = true;

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    input::mouse::AccumulatedMouseMotion,
    light::light_consts::lux,
    pbr::Atmosphere,
    post_process::bloom::Bloom,
    // post_process::motion_blur::MotionBlur, // heavy on GPU
    prelude::*,
    scene::SceneInstanceReady,
};
use bevy_rapier3d::prelude::*;
use std::{f32::consts::FRAC_PI_2, ops::Range};

mod aircraft_mechanics;
mod input;
use input::{GamepadSettings, Keymap};

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
                y: 4.0,
                z: 0.0,
            },
        }
    }
}

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct Aircraft;

#[derive(Resource)]
struct InputAxis {
    pitch: f32,    // Pitch
    yaw: f32,      // Yaw
    roll: f32,     // Roll
    throttle: f32, // Throttle
}

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig::default(),
        })
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .insert_resource(InputAxis {
            pitch: 0.,
            yaw: 0.,
            roll: 0.,
            throttle: 1.,
        })
        .insert_resource(GamepadSettings::default())
        .insert_resource(CameraSettings::default())
        .insert_resource(input::Keymap::default())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input::input_system,
                aircraft_mechanics::aircraft_mechanics,
                camera_movement,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_settings: Res<CameraSettings>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("aircraft.glb")),
    );

    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // landscape
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("landscapeFS.glb")),
    ));

    commands
        .spawn(Collider::cuboid(10., 1., 10.))
        .insert(Transform::from_xyz(0., -20., 0.));

    // aircraft
    commands
        .spawn((
            animation_to_play,
            SceneRoot(asset_server.load("aircraft.glb#Scene0")),
            Aircraft,
            RigidBody::Dynamic,
            Collider::ball(10.),
            Restitution::coefficient(0.2),
            Transform::from_xyz(0., 20., 0.),
            ExternalForce {
                force: Vec3::ZERO,
                torque: Vec3::ZERO,
            },
        ))
        .observe(play_animation_when_ready)
        .with_children(|parent| {
            parent.spawn((
                Camera3d::default(),
                Transform::from_translation(camera_settings.follow_default_position)
                    .looking_at(camera_settings.follow_default_lookat, Vec3::Y),
                Atmosphere::EARTH,
                Exposure::SUNLIGHT,
                Tonemapping::AgX,
                Bloom::NATURAL,
                // MotionBlur is heavy to compute, only do it if your computer is strong enough
                // MotionBlur {
                //     shutter_angle: 1.0,
                //     samples: 2,
                // },
                FollowCamera,
            ));
        });

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(2.0, 1.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn play_animation_when_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(animation_to_play) = animations_to_play.get(scene_ready.entity) {
        for child in children.iter_descendants(scene_ready.entity) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(animation_to_play.index).repeat();
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
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
