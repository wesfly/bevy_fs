pub mod animations;
pub mod buttons;
pub mod lights;
pub mod screens;

pub mod airfoils;
pub mod breeze;
mod helicopter;
mod j3cub;

use crate::{GameState, aircraft, camera::Camera, data_from_gltf::load, input::ControlInputs};
use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;

pub fn alpha_deg(velocity: &Vec3, transform: &Transform) -> f32 {
    let velocity = velocity.normalize_or_zero();
    let forward = transform.local_x();
    let right = transform.local_y();

    let sin = forward.cross(velocity).dot(right.as_vec3());
    let cos = forward.dot(velocity);

    -sin.atan2(cos).to_degrees()
}

#[derive(Debug, Deserialize, Component)]
pub enum RotorTypes {
    Main,
    Rear,
}

#[derive(Debug, Deserialize)]
pub struct Rotor {
    pub rotor: RotorTypes,
}

#[derive(Debug, Deserialize, Component)]
pub enum ControlSurfaces {
    CanardPort,
    CanardStarboard,
    Rudder,
    ElevatorPort,
    ElevatorStarboard,
    FlapPort,
    FlapStarboard,
    AileronPort,
    AileronStarboard,
}

#[derive(Debug, Deserialize)]
pub struct ControlSurface {
    pub control_surface: ControlSurfaces,
}

#[derive(Resource, Default, Deserialize, PartialEq, Clone, Copy)]
pub enum AircraftTypes {
    Helicopter,
    J3Cub,
    #[default]
    Breeze,
}

#[derive(Resource, Copy, Clone)]
pub struct EngineState {
    on: bool,
    throttle: f32, // 0 to 1
}

/// Angles in radians
#[derive(Resource, Copy, Clone)]
pub struct ControlSurfacesDeflection {
    canards: BothSides<f32>,
    aileron: BothSides<f32>,
    elevator: BothSides<f32>,
    rudder: f32,
    ground_brakes: f32, // 0 to 1
}

#[derive(Resource, Copy, Clone)]
pub struct AircraftState {
    pub control_surfaces: ControlSurfacesDeflection,

    pub aircraft_type: AircraftTypes,
    pub anti_col_lts_on: bool,
    pub pos_lts_on: bool,
    pub strobe_lts_on: bool,
    pub form_lts_on: bool,
    pub ldg_lts_on: bool,
    pub landing_gear_deployed: bool,
    pub parking_brake: bool,
    pub on_ground: bool,
    pub engine: EngineState,
}

impl Default for AircraftState {
    fn default() -> Self {
        AircraftState {
            control_surfaces: ControlSurfacesDeflection {
                canards: 0.0_f32.both_sides(),
                aileron: 0.0_f32.both_sides(),
                elevator: 0.0_f32.both_sides(),
                rudder: 0.0,

                ground_brakes: 0.0,
            },
            engine: EngineState {
                on: false,
                throttle: 0.0,
            },
            aircraft_type: AircraftTypes::Helicopter,
            anti_col_lts_on: false,
            pos_lts_on: false,
            strobe_lts_on: false,
            form_lts_on: false,
            ldg_lts_on: false,
            landing_gear_deployed: false,
            parking_brake: false,
            on_ground: false,
        }
    }
}

pub fn main(
    input: Res<ControlInputs>,
    mut state: ResMut<AircraftState>,
    gizmos: Gizmos,
    mut aircraft: Single<
        (
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
    spatial_query: SpatialQuery,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    if !game_state.running {
        return;
    }

    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            helicopter::mechanics(input, state, gizmos, &mut aircraft);
        }
        AircraftTypes::J3Cub => {
            // TODO
            aircraft::breeze::mechanics::mechanics(input, &mut state, &mut aircraft);
        }
        AircraftTypes::Breeze => {
            aircraft::breeze::mechanics::mechanics(input, &mut state, &mut aircraft);
            aircraft::breeze::landing_gear::spring_forces(spatial_query, aircraft, time, state);
        }
    }
}

#[derive(Component)]
pub struct Aircraft;

pub fn spawn_breeze(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AircraftState>,
) {
    state.aircraft_type = AircraftTypes::Breeze;
    state.engine.on = true;
    state.anti_col_lts_on = true;
    state.ldg_lts_on = true;

    let level = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    breeze::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 650.0, 0.0).with_rotation(level),
        asset_server,
        Vec3::new(100.0, 0.0, 0.0),
    );
    Camera::spawn(&mut commands);
}

pub fn spawn_j3cub(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AircraftState>,
) {
    state.aircraft_type = AircraftTypes::J3Cub;
    state.engine.on = true;
    // state.anti_col_lts_on = true;
    // state.ldg_lts_on = true;

    Camera::spawn(&mut commands);

    let level = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    j3cub::spawn(
        &mut commands,
        Transform::from_xyz(0.0, 1000.0, 0.0).with_rotation(level),
        asset_server,
    );
}

pub fn spawn_helicopter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AircraftState>,
) {
    state.aircraft_type = AircraftTypes::Helicopter;

    let path = "aircraft/helicopter/helicopter.gltf";

    let spawn_pos = Vec3::new(0.0, 100.0, 0.0);
    let spawn_vel = Vec3::new(0.0, 0.0, 0.0);

    // Aircraft model
    commands
        .spawn((
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
            Transform::from_translation(spawn_pos),
            Mass(10_000.0),
            LinearVelocity(spawn_vel),
        ))
        .observe(load);

    Camera::spawn(&mut commands);
}

#[derive(Clone, Copy, Debug)]
pub struct BothSides<T> {
    pub port: T,
    pub starboard: T,
}

pub trait BothSidesExt {
    fn both_sides(self) -> BothSides<f32>;
}

impl BothSidesExt for f32 {
    fn both_sides(self) -> BothSides<f32> {
        BothSides {
            port: self,
            starboard: self,
        }
    }
}
