pub mod animations;
pub mod buttons;
pub mod lights;
pub mod screens;

pub mod airfoils;
pub mod breeze;
mod helicopter;
mod j3cub;

use crate::{
    EARTH_RADIUS, GameState, Settings, absolute_position, aircraft, camera::AircraftCamera,
    data_from_gltf::load, input::ControlInputs, scenery::terrain::coord_to_pos,
};
use avian3d::prelude::*;
use bevy::{
    dev_tools::diagnostics_overlay::DiagnosticsOverlay,
    math::{DMat3, DQuat, DVec3},
    prelude::*,
    world_serialization::WorldInstanceReady,
};
use big_space::prelude::*;
use serde::Deserialize;

pub fn alpha_deg(velocity: &DVec3, transform: &Transform) -> f64 {
    let velocity = velocity.normalize_or_zero();
    let forward = transform.local_x();
    let right = transform.local_y();

    let sin = forward.cross(velocity.as_vec3()).dot(right.as_vec3());
    let cos = forward.dot(velocity.as_vec3());

    -sin.atan2(cos).to_degrees() as f64
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
            &Position,
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
    root_grid: Single<(Entity, &Grid), With<BigSpace>>,
    settings: Res<Settings>,
) {
    let (root_grid_id, grid) = *root_grid;
    commands.spawn(DiagnosticsOverlay::fps());
    state.aircraft_type = AircraftTypes::Breeze;
    state.engine.on = true;
    state.ldg_lts_on = true;

    let normalized_pos = coord_to_pos(settings.terrain.coord);

    let altitude = 1000.0;
    let translation =
        normalized_pos.as_dvec3() * EARTH_RADIUS as f64 + normalized_pos.as_dvec3() * altitude;
    let (object_cell, object_pos) = grid.translation_to_grid(translation);
    let surface_up = translation.normalize();
    let level = DQuat::from_rotation_arc(DVec3::Y, surface_up)
        * DQuat::from_rotation_x(core::f64::consts::FRAC_PI_2);

    let up = surface_up;
    let reference_forward = DVec3::NEG_Z;
    let forward = (reference_forward - up * reference_forward.dot(up)).normalize();
    let right = forward.cross(up).normalize();
    let up = right.cross(forward);

    let rotation = DQuat::from_mat3(&DMat3::from_cols(right, up, -forward));
    commands.spawn((
        AircraftCamera::spawn(),
        ChildOf(root_grid_id),
        object_cell,
        Transform::from_translation(object_pos).with_rotation(rotation.as_quat()),
    ));

    breeze::spawn(
        &mut commands,
        Transform::from_translation(object_pos).with_rotation(level.as_quat()),
        asset_server,
        Rotation(level),
        DVec3::new(100.0, 0.0, 0.0),
        object_cell,
        root_grid_id,
        translation,
    );
}

pub fn spawn_j3cub(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AircraftState>,
    root_grid: Single<(Entity, &Grid), With<BigSpace>>,
    settings: Res<Settings>,
) {
    state.aircraft_type = AircraftTypes::J3Cub;
    state.engine.on = true;

    let (root_grid_id, grid) = *root_grid;
    commands.spawn(DiagnosticsOverlay::fps());

    let translation =
        coord_to_pos(settings.terrain.coord) * EARTH_RADIUS + Vec3::new(0.0, 1000.0, 0.0);
    let (object_cell, object_pos) = grid.translation_to_grid(translation);
    let up = translation.normalize();
    let level =
        Quat::from_rotation_arc(Vec3::Y, up) * Quat::from_rotation_x(core::f32::consts::FRAC_PI_2);

    commands.spawn((
        AircraftCamera::spawn(),
        ChildOf(root_grid_id),
        object_cell,
        Transform::from_translation(object_pos)
            .with_rotation(Quat::from_rotation_z(core::f32::consts::PI)),
    ));
    j3cub::spawn(
        &mut commands,
        Transform::from_translation(object_pos).with_rotation(level),
        asset_server,
        translation.as_dvec3(),
        object_cell,
        root_grid_id,
        Rotation(level.as_dquat()),
    );
}

pub fn spawn_helicopter(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut state: ResMut<AircraftState>,
    root_grid: Single<(Entity, &Grid), With<BigSpace>>,
    settings: Res<Settings>,
) {
    state.aircraft_type = AircraftTypes::Helicopter;
    let (root_grid_id, grid) = *root_grid;
    commands.spawn(DiagnosticsOverlay::fps());

    let translation =
        coord_to_pos(settings.terrain.coord) * EARTH_RADIUS + Vec3::new(0.0, 1000.0, 0.0);
    let (object_cell, object_pos) = grid.translation_to_grid(translation);
    let up = translation.normalize();
    let level = Quat::from_rotation_arc(Vec3::Y, up);

    let path = "aircraft/helicopter/helicopter.gltf";

    commands.spawn((
        AircraftCamera::spawn(),
        ChildOf(root_grid_id),
        object_cell,
        Transform::from_translation(object_pos)
            .with_rotation(Quat::from_rotation_z(core::f32::consts::PI)),
    ));

    commands
        .spawn((
            object_cell,
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Aircraft,
            RigidBody::Dynamic,
            Transform::from_translation(object_pos).with_rotation(level),
            Position(absolute_position(&object_cell, object_pos)),
            Rotation(level.as_dquat()),
            Mass(10_000.0),
            ChildOf(root_grid_id),
        ))
        .observe(|trigger: On<WorldInstanceReady>, mut commands: Commands| {
            commands
                .entity(trigger.entity.entity())
                .insert(ColliderConstructorHierarchy::new(
                    ColliderConstructor::ConvexHullFromMesh,
                ));
        })
        .observe(load);
}

#[derive(Clone, Copy, Debug)]
pub struct BothSides<T> {
    pub port: T,
    pub starboard: T,
}

pub trait BothSidesF32 {
    fn both_sides(self) -> BothSides<f32>;
}
impl BothSidesF32 for f32 {
    fn both_sides(self) -> BothSides<f32> {
        BothSides {
            port: self,
            starboard: self,
        }
    }
}
