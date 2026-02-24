/*
I made a little flight simulator here. Check out the README for further information.
If you have fixes or want to contribute, just make a pull request.

I don't exactly know where these InheritedVisibility warnings are coming from (they're probably from the aircraft and its hitbox), I'm just
ignoring them.

TODO: Add big_space someday
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
        ACOL_OFF_DURATION, AircraftState, AircraftTypes, LightsTimers, STROBE_OFF_DURATION,
        update_light_cycle, update_lights, update_mesh_lights, update_rotors,
    },
    camera::{CameraSettings, camera_controller},
    input::InputAxis,
    sse::{insert_sse_resources, sse_config},
    terrain::{Terrain, TerrainData},
    ui::{MenuCamera, UI},
};
use avian3d::prelude::*;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    ecs::system::SystemId,
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    pbr::ExtendedMaterial,
    post_process::motion_blur::MotionBlur,
    prelude::*,
};
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Resource, PartialEq)]
pub enum GameState {
    Running,
    Menu,
}

#[derive(Resource, Deserialize)]
pub struct Settings {
    aircraft: AircraftTypes,
    gamepad: input::Gamepad,
    motion_blur_enabled: bool,
    shadow_distance: f32,
    screen_space_effects: bool,
    terrain: Terrain,
    sun_position: Vec3,
}

impl Settings {
    fn fetch() -> Self {
        let json_data = fs::read_to_string("settings.json")
            .expect("Try running 'cargo run (--release)' from the project root folder.");
        let settings: Self =
            serde_json::from_str(&json_data).expect("Failed to serialize settings file");
        settings
    }
}

#[derive(Resource)]
pub struct RunOnceSystemList(HashMap<String, SystemId>);

impl FromWorld for RunOnceSystemList {
    fn from_world(world: &mut World) -> Self {
        let mut my_item_systems = RunOnceSystemList(HashMap::new());
        my_item_systems
            .0
            .insert("setup_scene".into(), world.register_system(setup_scene));
        my_item_systems.0.insert(
            "setup_terrain".into(),
            world.register_system(terrain::spawn_terrain),
        );
        my_item_systems.0.insert(
            "setup_aircraft".into(),
            world.register_system(aircraft::spawn),
        );

        my_item_systems
    }
}

fn main() {
    let settings = Settings::fetch();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(UI)
        // Resources
        .insert_resource(InputAxis {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            throttle: 0.0,
        })
        .insert_resource(GameState::Menu)
        .insert_resource(CameraSettings::default())
        .insert_resource(input::Keymap::default())
        .insert_resource(Settings::fetch())
        .insert_resource(LightsTimers {
            acol: Timer::from_seconds(ACOL_OFF_DURATION, TimerMode::Repeating),
            acol_on_cycle: false,
            strobe: Timer::from_seconds(STROBE_OFF_DURATION, TimerMode::Repeating),
            strobe_on_cycle: false,
        })
        .init_resource::<RunOnceSystemList>()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AircraftState {
            engine_on: true,
            anti_col_lts_on: true,
            pos_lts_on: true,
            strobe_lts_on: true,
            aircraft_type: settings.aircraft,
        })
        .insert_resource(TerrainData(Vec::new()))
        // Systems
        .add_systems(
            Update,
            (
                input::input_system,
                aircraft::mechanics::mechanics,
                camera_controller,
                update_light_cycle,
                update_rotors,
                (update_mesh_lights, update_lights).after(update_light_cycle),
            ),
        );

    if settings.screen_space_effects {
        insert_sse_resources(&mut app);
    }

    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            frame_time_graph_config: FrameTimeGraphConfig {
                enabled: true,
                min_fps: 40.0,
                target_fps: 100.0,
            },
            ..default()
        },
    });

    app.run();
}

fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    water_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, sse::Water>>>>,
    camera: Single<Entity, With<MenuCamera>>,
) {
    commands.entity(*camera).despawn();

    if let Some(material) = water_materials {
        sse::spawn_water(&mut commands, &asset_server, &mut meshes, material);
    }

    commands.spawn((
        SceneRoot(asset_server.load("hospital.glb#Scene0")),
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: settings.shadow_distance,
        ..Default::default()
    }
    .build();

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
