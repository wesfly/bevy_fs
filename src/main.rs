/*
I made a little flight simulator here. Check out the README for further information.
If you have fixes or want to contribute, just make a pull request.

TODO: Add big_space someday

Spatial units are in metres
Default for speed is m/s
*/

pub const METRES_TO_FEET: f32 = 3.28084;
pub const M_S_TO_KTS: f32 = 1.943844;

mod aircraft;
mod camera;
mod data_from_gltf;
mod input;
mod scenery;
mod sse;
mod ui;

use crate::{
    aircraft::{
        AircraftState,
        animations::{update_control_surfaces, update_rotors},
        breeze::landing_gear::{LandingGear, LandingGearCommand, LandingGearStatus},
        buttons::{Button, ButtonMessages},
        lights::{ACOL_OFF_DURATION, Light, LightsTimers, STROBE_OFF_DURATION},
    },
    camera::{Camera, CameraPosition, CameraSettings, rotate_sun},
    input::ControlInputs,
    scenery::terrain::{
        Chunk, ChunkMessage, LoadedChunks, TerrainMaterial, TerrainSettings, dynamic_chunks,
        poll_terrain,
    },
    sse::Sse,
    ui::{Menu, UI},
};
use avian3d::prelude::*;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    ecs::system::SystemId,
    pbr::ExtendedMaterial,
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    window::{CursorIcon, SystemCursorIcon},
};
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[allow(unused)] // AircraftFdmDebugPlugin, FdmGizmos can be commented out in the main function
use avian_fdm::{
    plugin::AircraftFdmPlugin,
    prelude::{AircraftFdmDebugPlugin, FdmGizmos, ShowColliders},
};

pub fn bevy_to_aerospace_coords() -> Quat {
    Quat::from_mat3(&Mat3::from_cols(
        Vec3::Y,  // Camera X goes to Aerospace +Y (Right)
        -Vec3::Z, // Camera Y goes to Aerospace -Z (Up)
        -Vec3::X, // Camera Z goes to Aerospace -X
    ))
}

#[derive(Resource, Deserialize)]
pub struct Settings {
    gamepad: input::Gamepad,
    shadow_distance: f32,
    terrain: TerrainSettings,
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
pub struct GameState {
    pub in_menu: bool,
    pub running: bool,
}

#[derive(Resource)]
pub struct RunOnceSystemList(HashMap<String, SystemId>);

impl FromWorld for RunOnceSystemList {
    fn from_world(world: &mut World) -> Self {
        let mut run_once_systems = RunOnceSystemList(HashMap::new());
        run_once_systems.0.insert(
            "setup_scene".into(),
            world.register_system(scenery::setup_scene),
        );
        run_once_systems.0.insert(
            "setup_breeze".into(),
            world.register_system(aircraft::spawn_breeze),
        );
        run_once_systems.0.insert(
            "setup_j3cub".into(),
            world.register_system(aircraft::spawn_j3cub),
        );
        run_once_systems.0.insert(
            "spawn_helicopter".into(),
            world.register_system(aircraft::spawn_helicopter),
        );
        run_once_systems.0.insert(
            "spawn_ui_hud".into(),
            world.register_system(UI::setup_ui_hud),
        );

        run_once_systems
            .0
            .insert("spawn_menu".into(), world.register_system(Menu::spawn));

        run_once_systems
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(MeshPickingPlugin)
        .add_plugins(AircraftFdmPlugin::default())
        // .insert_gizmo_config(
        //     FdmGizmos {
        //         force_scale: 1.0 / 600.0,
        //         total_force_color: None,
        //         weight_color: None,
        //         ..FdmGizmos::default()
        //     },
        //     GizmoConfig::default(),
        // )
        // .add_plugins(AircraftFdmDebugPlugin)
        // .add_plugins(PhysicsDebugPlugin)
        .add_plugins(UI)
        .add_plugins(Sse)
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                frame_time_graph_config: FrameTimeGraphConfig {
                    enabled: true,
                    min_fps: 40.0,
                    target_fps: 100.0,
                },
                ..default()
            },
        })
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterial>,
        >::default())
        // Resources
        .insert_resource(PhysicsPickingSettings {
            require_markers: true,
        })
        .insert_resource(GameState {
            running: false,
            in_menu: false,
        })
        .insert_resource(ControlInputs {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            throttle: 0.0,
            ground_brakes: 0.0,
        })
        .init_resource::<LoadedChunks>()
        .init_resource::<ShowColliders>()
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
        .insert_resource(AircraftState::default())
        .insert_resource(LandingGearStatus::Retracted)
        .insert_resource(CameraPosition(Transform {
            translation: Vec3::ZERO,
            ..default()
        }))
        // Messages
        .add_message::<LandingGearCommand>()
        .add_message::<ChunkMessage>()
        .add_message::<ButtonMessages>()
        // Systems
        .add_systems(
            Update,
            (
                update_rotors,
                Camera::controller,
                input::input_system,
                aircraft::buttons::update_cursor,
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                screenshot,
                update_control_surfaces,
                Light::update_light_cycle,
                (Light::update_mesh_lights, Light::update_lights).after(Light::update_light_cycle),
                LandingGear::operate_landing_gear,
                screenshot_saving,
                aircraft::screens::update_screens,
                aircraft::main,
                poll_terrain,
                Chunk::message_reader,
                Button::listener,
                rotate_sun,
                dynamic_chunks,
                game_state,
            ),
        );
    app.run();
}

fn screenshot(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::F3) {
        let now = chrono::Local::now();
        let path = format!("./screenshots/user/screenshot-{:?}.png", now);
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
    }
}

fn screenshot_saving(
    mut commands: Commands,
    screenshot_saving: Query<Entity, With<Capturing>>,
    window: Single<Entity, With<Window>>,
) {
    match screenshot_saving.iter().count() {
        0 => {
            commands.entity(*window).remove::<CursorIcon>();
        }
        x if x > 0 => {
            commands
                .entity(*window)
                .insert(CursorIcon::from(SystemCursorIcon::Progress));
        }
        _ => {}
    }
}

fn game_state(mut physics_time: ResMut<Time<Physics>>, game_state: Res<GameState>) {
    match game_state.running {
        false => physics_time.pause(),
        true => physics_time.unpause(),
    }
}
