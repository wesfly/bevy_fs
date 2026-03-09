pub mod animations;
pub mod landing_gear;
pub mod mechanics;

use crate::{
    Settings,
    data_from_gltf::{InterfaceOperation, InterfaceType, Lights, load},
    motion_blur,
};
use avian3d::prelude::*;
use bevy::{
    anti_alias::fxaa::Fxaa,
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    light::AtmosphereEnvironmentMapLight,
    pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use serde::Deserialize;
use std::time::Duration;

pub const STROBE_OFF_DURATION: f32 = 1.0;
pub const STROBE_ON_DURATION: f32 = 0.1;
pub const ACOL_OFF_DURATION: f32 = 1.2;
pub const ACOL_ON_DURATION: f32 = 0.1;

#[derive(Resource, Default, Deserialize)]
pub enum AircraftTypes {
    Helicopter,
    #[default]
    Aeroplane,
}

#[derive(Resource)]
pub struct AircraftState {
    pub aircraft_type: AircraftTypes,
    pub engine_on: bool,
    pub anti_col_lts_on: bool,
    pub pos_lts_on: bool,
    pub strobe_lts_on: bool,
    pub landing_gear_deployed: bool,
}

impl Default for AircraftState {
    fn default() -> Self {
        AircraftState {
            aircraft_type: AircraftTypes::Helicopter,
            engine_on: false,
            anti_col_lts_on: false,
            pos_lts_on: false,
            strobe_lts_on: false,
            landing_gear_deployed: false,
        }
    }
}

#[derive(Component)]
pub struct Aircraft;

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&crate::data_from_gltf::Button>,
    mut transform: Query<&mut Transform, With<crate::data_from_gltf::Button>>,
    mut state: ResMut<AircraftState>,
) {
    let button = function_comps.get(press.entity.entity()).unwrap();
    let bool;
    match button.operation.as_ref().unwrap() {
        InterfaceOperation::Engine => {
            bool = Some(state.engine_on);
            state.engine_on = !state.engine_on
        }
        InterfaceOperation::AntiColLt => {
            bool = Some(state.anti_col_lts_on);
            state.anti_col_lts_on = !state.anti_col_lts_on
        }
        InterfaceOperation::PositionLt => {
            bool = Some(state.pos_lts_on);
            state.pos_lts_on = !state.pos_lts_on
        }
        InterfaceOperation::StrobeLt => {
            bool = Some(state.strobe_lts_on);
            state.strobe_lts_on = !state.strobe_lts_on
        }
        _ => bool = None,
    }

    const SWITCH_ANGLE_LIMIT: f32 = 70.0;
    if let InterfaceType::Switch = button.interface_type
        && let Some(mut bool) = bool
    {
        if let Some(inverse) = button.inverse
            && inverse
        {
            bool = !bool
        }

        let angle = match bool {
            true => -SWITCH_ANGLE_LIMIT,
            false => SWITCH_ANGLE_LIMIT,
        };
        transform
            .get_mut(press.entity.entity())
            .unwrap()
            .rotate_local_x(angle.to_radians());
    }
}

#[derive(Resource)]
pub struct LightsTimers {
    pub acol: Timer,
    pub acol_on_cycle: bool,
    pub strobe: Timer,
    pub strobe_on_cycle: bool,
}

pub fn update_light_cycle(time: Res<Time>, mut timer: ResMut<LightsTimers>) {
    let delta = time.delta();
    if timer.acol.just_finished() && !timer.acol_on_cycle {
        timer.acol_on_cycle = true;
        timer
            .acol
            .set_duration(Duration::from_secs_f32(ACOL_ON_DURATION));
    } else if timer.acol.just_finished() && timer.acol_on_cycle {
        timer.acol_on_cycle = false;
        timer
            .acol
            .set_duration(Duration::from_secs_f32(ACOL_OFF_DURATION));
    }

    if timer.strobe.just_finished() && !timer.strobe_on_cycle {
        timer.strobe_on_cycle = true;
        timer
            .strobe
            .set_duration(Duration::from_secs_f32(STROBE_ON_DURATION));
    } else if timer.strobe.just_finished() && timer.strobe_on_cycle {
        timer.strobe_on_cycle = false;
        timer
            .strobe
            .set_duration(Duration::from_secs_f32(STROBE_OFF_DURATION));
    }

    timer.acol.tick(delta);
    timer.strobe.tick(delta);
}

pub fn update_mesh_lights(
    material_handles: Query<(&MeshMaterial3d<StandardMaterial>, &Lights, Entity), With<Lights>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<AircraftState>,
    timer: ResMut<LightsTimers>,
) {
    #[allow(irrefutable_let_patterns)] // Acting like I know what I'm doing
    for material_handle in material_handles.iter() {
        if let Some(material) = materials.get_mut(material_handle.0)
            && let LinearRgba {
                ref mut red,
                ref mut green,
                ref mut blue,
                alpha: _,
            } = material.emissive
        {
            match material_handle.1 {
                Lights::AntiCol => {
                    if state.anti_col_lts_on && timer.acol_on_cycle {
                        *red = 100.0
                    } else {
                        *red = 0.0
                    }
                }
                Lights::PositionPort => {
                    if state.pos_lts_on {
                        *red = 100.
                    } else {
                        *red = 0.0
                    }
                }
                Lights::PositionStarboard => {
                    if state.pos_lts_on {
                        *green = 100.
                    } else {
                        *green = 0.0
                    }
                }
                Lights::PositionRear => {
                    if state.pos_lts_on {
                        *red = 100.;
                        *green = 100.;
                        *blue = 100.
                    } else {
                        *red = 0.;
                        *green = 0.;
                        *blue = 0.
                    }
                }
                Lights::Strobe => {
                    if state.strobe_lts_on && timer.strobe_on_cycle {
                        *red = 100.;
                        *green = 100.;
                        *blue = 100.
                    } else {
                        *red = 0.;
                        *green = 0.;
                        *blue = 0.
                    }
                }
            }
        }
    }
}

pub fn update_lights(
    state: Res<AircraftState>,
    timer: ResMut<LightsTimers>,
    query: Query<(&mut PointLight, &Lights)>,
) {
    for (mut point_light, light) in query {
        let (colour, on) = match light {
            Lights::PositionPort => (Color::linear_rgb(1.0, 0.0, 0.0), state.pos_lts_on),
            Lights::PositionStarboard => (Color::linear_rgb(0.0, 1.0, 0.0), state.pos_lts_on),
            Lights::PositionRear => (Color::linear_rgb(1.0, 1.0, 1.0), state.pos_lts_on),

            Lights::AntiCol => (
                Color::linear_rgb(1.0, 0.0, 0.0),
                (state.anti_col_lts_on && timer.acol_on_cycle),
            ),

            Lights::Strobe => (
                Color::linear_rgb(1.0, 1.0, 1.0),
                (state.strobe_lts_on && timer.strobe_on_cycle),
            ),
        };

        point_light.color = colour;

        if on {
            point_light.intensity = 10000.0;
        } else {
            point_light.intensity = 0.0
        }
    }
}

pub fn spawn_aeroplane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    settings: Res<Settings>,
    mut state: ResMut<AircraftState>,
) {
    let path = "aeroplane.glb";

    state.aircraft_type = AircraftTypes::Aeroplane;
    state.engine_on = true;
    state.pos_lts_on = true;
    state.anti_col_lts_on = true;
    state.strobe_lts_on = true;

    // Aircraft model
    commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            Transform {
                translation: Vec3 {
                    x: 0.0,
                    y: 500.0,
                    z: 0.0,
                },
                rotation: Quat::from_rotation_y(-90.0_f32.to_degrees()),
                ..default()
            },
            Mass(10_000.0),
            SweptCcd::new_with_mode(SweepMode::NonLinear),
            LinearVelocity(Vec3::new(-100.0, 0.0, 0.0)),
            children![
                (
                    Collider::capsule(0.5, 1.0),
                    Transform::from_xyz(1.2, -0.5, 2.0),
                    Friction::new(0.0),
                    Mass(0.0),
                    Name::new("rear left"),
                ),
                (
                    Collider::capsule(0.5, 1.0),
                    Transform::from_xyz(-1.2, -0.5, 2.0),
                    Friction::new(0.0),
                    Mass(0.0),
                    Name::new("rear right"),
                ),
                (
                    Collider::capsule(0.5, 1.0),
                    Transform::from_xyz(0.0, -0.56, -2.8),
                    Friction::new(0.0),
                    Mass(0.0),
                    Name::new("nosewheel"),
                )
            ],
        ))
        .observe(load);

    let mut camera = commands.spawn((
        Camera3d::default(),
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
        Fxaa::default(),
        Msaa::Off,
        ScreenSpaceReflections::default(),
        Hdr,
        crate::camera::Camera,
    ));

    if let Some(mb) = motion_blur(&settings) {
        camera.insert(mb);
    }
}

pub fn spawn_helicopter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    settings: Res<Settings>,
    mut state: ResMut<AircraftState>,
) {
    state.aircraft_type = AircraftTypes::Helicopter;

    let path = "helicopter.glb";

    let (spawn_pos, spawn_vel) = (Vec3::new(0.0, 12.0, 0.0), Vec3::new(0.0, 0.0, 0.0));

    // Aircraft model
    commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            Transform::from_translation(spawn_pos),
            Mass(10_000.0),
            LinearVelocity(spawn_vel),
        ))
        .observe(load);

    let mut camera = commands.spawn((
        Camera3d::default(),
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
        crate::camera::Camera,
        // SSR
        ScreenSpaceReflections::default(),
        Msaa::Off,
        Fxaa::default(),
    ));

    if let Some(mb) = motion_blur(&settings) {
        camera.insert(mb);
    }
}
