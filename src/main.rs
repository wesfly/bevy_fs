/*
I made a little flight simulator here. Check out the README for further information.
If you have fixes or want to contribute, just make a pull request.

TODO: Add big_space someday
*/

mod aircraft;
mod camera;
mod data_from_gltf;
mod input;
mod scenery;
mod sse;
mod ui;

use crate::{
    aircraft::{
        ACOL_OFF_DURATION, AircraftState, LightsTimers, STROBE_OFF_DURATION,
        animations::{update_control_surfaces, update_rotors},
        landing_gear::{self, LandingGearCommand, LandingGearStatus},
        update_light_cycle, update_lights, update_mesh_lights,
    },
    camera::{CameraPosition, CameraSettings, camera_controller},
    input::InputAxis,
    scenery::terrain::{Terrain, TerrainData},
    sse::SSE,
    ui::UI,
};
use avian3d::prelude::*;
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    ecs::system::SystemId,
    post_process::motion_blur::MotionBlur,
    prelude::*,
    render::view::screenshot::{Capturing, Screenshot, save_to_disk},
    window::{CursorIcon, SystemCursorIcon},
};
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Resource, Deserialize)]
pub struct Settings {
    gamepad: input::Gamepad,
    motion_blur_enabled: bool,
    shadow_distance: f32,
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
        my_item_systems.0.insert(
            "setup_scene".into(),
            world.register_system(scenery::setup_scene),
        );
        my_item_systems.0.insert(
            "setup_terrain".into(),
            world.register_system(scenery::terrain::spawn_terrain),
        );
        my_item_systems.0.insert(
            "setup_aeroplane".into(),
            world.register_system(aircraft::spawn_aeroplane),
        );
        my_item_systems.0.insert(
            "spawn_helicopter".into(),
            world.register_system(aircraft::spawn_helicopter),
        );
        my_item_systems.0.insert(
            "spawn_ui_hud".into(),
            world.register_system(ui::setup_ui_hud),
        );

        my_item_systems
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        // .add_plugins(PhysicsDebugPlugin)
        .add_plugins(MeshPickingPlugin)
        .add_plugins(UI)
        .add_plugins(SSE)
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
        // Resources
        .insert_resource(InputAxis {
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            throttle: 0.0,
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
        .init_resource::<RunOnceSystemList>()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AircraftState::default())
        .insert_resource(TerrainData(Vec::new()))
        .insert_resource(LandingGearStatus::Retracted)
        .insert_resource(CameraPosition(Transform {
            translation: Vec3::ZERO,
            ..default()
        }))
        // Messages
        .add_message::<LandingGearCommand>()
        // Systems
        .add_systems(
            Update,
            (
                input::input_system,
                update_light_cycle,
                update_rotors,
                update_control_surfaces,
                camera_controller,
                (update_mesh_lights, update_lights).after(update_light_cycle),
            ),
        )
        .add_systems(
            FixedUpdate,
            (
                screenshot,
                landing_gear::LandingGear::operate_landing_gear,
                screenshot_saving,
                aircraft::mechanics::mechanics,
            ),
        );

    app.run();
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

fn screenshot(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::F3) {
        let now = chrono::Local::now();
        let path = format!("./screenshots/user/screenshot-{:?}.png", now);
        info!("{now:?}");
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
