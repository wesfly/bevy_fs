pub mod animations;
pub mod landing_gear;
pub mod lights;
pub mod mechanics;

use crate::{
    Settings,
    data_from_gltf::{InterfaceOperation, InterfaceType, load},
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
    pub form_lts_on: bool,
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
            form_lts_on: false,
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
    if press.button == PointerButton::Primary {
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
}

pub fn spawn_aeroplane(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    settings: Res<Settings>,
    mut state: ResMut<AircraftState>,
) {
    let path = "models/aeroplane/aeroplane.gltf";

    state.aircraft_type = AircraftTypes::Aeroplane;
    state.engine_on = true;
    state.pos_lts_on = true;
    state.anti_col_lts_on = true;
    state.strobe_lts_on = true;
    state.form_lts_on = true;

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

    let path = "models/helicopter/helicopter.gltf";

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
