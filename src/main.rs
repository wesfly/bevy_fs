/*
I made a little flight simulator here. Check out the README for further information.
If you have fixes or want to contribute, just make a pull request (unless it's AI-generated)

I don't exactly know where these InheritedVisibility warnings are coming from (they're probably from the aircraft and its hitbox), I'm just
ignoring them.
*/

mod aircraft;
mod camera;
mod data_from_gltf;
mod input;
mod sse;
mod terrain;
mod ui;

use crate::{
    aircraft::{
        ACOL_OFF_DURATION, AircraftState, LightsTimers, STROBE_OFF_DURATION, update_lights,
    },
    camera::{CameraSettings, camera_controller},
    data_from_gltf::{buttons_from_gltf, lights_from_gltf},
    sse::{insert_sse_resources, sse_config},
    terrain::TerrainData,
    ui::UI,
};

use avian3d::prelude::*;

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    light::{AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, light_consts::lux},
    pbr::{Atmosphere, AtmosphereSettings, ExtendedMaterial, ScatteringMedium},
    post_process::{bloom::Bloom, motion_blur::MotionBlur},
    prelude::*,
    render::view::Hdr,
    scene::SceneInstanceReady,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[cfg(debug_assertions)]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

#[derive(Serialize, Deserialize)]
struct Gamepad {
    enabled: bool,
}

#[derive(Resource, Serialize, Deserialize)]
pub struct Settings {
    gamepad: Gamepad,
    motion_blur_enabled: bool,
    shadow_distance: f32,
    screen_space_effects: bool,
    sun_position: Vec3,
}

impl Settings {
    fn fetch() -> Self {
        let json_data = fs::read_to_string("settings.json")
            .expect("Try running 'cargo run (--release)' from the project root folder.");
        let settings: Self = serde_json::from_str(&json_data).unwrap();
        settings
    }
}

#[derive(Component)]
struct Camera;

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
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(MeshPickingPlugin)
        .insert_resource(InputAxis {
            pitch: 0.,
            yaw: 0.,
            roll: 0.,
            throttle: 1.,
        })
        .insert_resource(CameraSettings::default())
        .insert_resource(input::Keymap::default())
        .insert_resource(Settings::fetch())
        .insert_resource(LightsTimers {
            acol: Timer::from_seconds(ACOL_OFF_DURATION, TimerMode::Repeating),
            acol_on_cycle: false,
            strobe: Timer::from_seconds(STROBE_OFF_DURATION, TimerMode::Repeating),
            strobe_on_cycle: false,
        })
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AircraftState::default())
        .insert_resource(TerrainData(Vec::new()))
        .add_plugins(UI)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input::input_system,
                aircraft::mechanics,
                camera_controller,
                update_lights,
            ),
        );

    let settings = Settings::fetch();
    if settings.screen_space_effects {
        insert_sse_resources(&mut app);
    }

    #[cfg(debug_assertions)]
    app.add_plugins(FpsOverlayPlugin::default());

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_settings: Res<CameraSettings>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    water_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, sse::Water>>>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    materials: ResMut<Assets<StandardMaterial>>,
    terrain_data: ResMut<TerrainData>,
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

    if let Some(abc) = water_materials {
        sse::spawn_water(&mut commands, &asset_server, &mut meshes, abc);
    }

    terrain::spawn_terrain(&mut commands, meshes, materials, terrain_data);

    // Aircraft collider
    let aircraft = commands
        .spawn((
            SceneRoot(asset_server.load("aircraft.glb#Scene1")),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            Transform::from_xyz(0., 20., 0.),
            Mass(5000.),
            Visibility::Hidden,
        ))
        .id();

    // The real aircraft model
    commands
        .spawn((
            SceneRoot(asset_server.load("aircraft.glb#Scene0")),
            Visibility::Visible,
            ChildOf(aircraft),
            animation_to_play,
        ))
        .observe(buttons_from_gltf)
        .observe(lights_from_gltf)
        .observe(play_animation_when_ready);

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: settings.shadow_distance,
        ..Default::default()
    }
    .build();

    let mut camera = commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_settings.follow_default_position)
            .looking_at(camera_settings.follow_default_lookat, Vec3::Y),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereEnvironmentMapLight::default(),
        AtmosphereSettings::default(),
        Exposure::SUNLIGHT,
        Tonemapping::AgX,
        Bloom::NATURAL,
        Projection::from(PerspectiveProjection {
            fov: 50.0_f32.to_radians(),
            ..default()
        }),
        Hdr,
        Camera,
        ChildOf(aircraft),
    ));

    if let Some(sse) = sse_config(&settings) {
        camera.insert(sse);
    }

    if let Some(a) = motion_blur(&settings) {
        camera.insert(a);
    }

    let sun_position = settings.sun_position;
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_translation(sun_position).looking_at(Vec3::ZERO, Vec3::Y),
        cascade,
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

fn motion_blur(settings: &Res<Settings>) -> Option<MotionBlur> {
    if settings.motion_blur_enabled {
        Some(MotionBlur {
            shutter_angle: 0.5,
            samples: 2,
        })
    } else {
        None
    }
}
