pub mod mechanics;

use crate::{
    Settings,
    data_from_gltf::{InterfaceOperation, InterfaceType, Lights, RotorTypes, load},
    motion_blur,
};
use avian3d::prelude::*;
use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    light::AtmosphereEnvironmentMapLight,
    pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium},
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

#[derive(Resource, Default, Deserialize, Clone)]
pub enum AircraftTypes {
    Helicopter,
    #[default]
    Aeroplane,
}

#[derive(Resource)]
pub struct AircraftState {
    pub engine_on: bool,
    pub anti_col_lts_on: bool,
    pub pos_lts_on: bool,
    pub strobe_lts_on: bool,
    pub aircraft_type: AircraftTypes,
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

pub fn update_rotors(
    query: Query<(&mut Transform, &RotorTypes)>,
    state: Res<AircraftState>,
    time: Res<Time>,
) {
    if state.engine_on {
        for (mut rotor, rotor_type) in query {
            match rotor_type {
                RotorTypes::Main => rotor.rotate_local_y(100.0 * time.delta_secs()),
                RotorTypes::Rear => rotor.rotate_local_z(100.0 * time.delta_secs()),
            }
        }
    }
}

pub fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    settings: Res<Settings>,
    state: Res<AircraftState>,
) {
    let path = match state.aircraft_type {
        AircraftTypes::Aeroplane => "aeroplane.glb",
        AircraftTypes::Helicopter => "helicopter.glb",
    };

    // Aircraft model
    let aircraft = commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            Transform::from_xyz(0., 2000., 0.),
            Mass(10_000.0),
            LinearVelocity(Vec3 {
                x: 0.0,
                y: 0.0,
                z: -100.0,
            }),
        ))
        .observe(load)
        .id();

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
        ChildOf(aircraft),
    ));

    // TODO make this a plugin
    if let Some(sse) = crate::sse_config(&settings) {
        camera.insert(sse);
    }

    if let Some(mb) = motion_blur(&settings) {
        camera.insert(mb);
    }
}
