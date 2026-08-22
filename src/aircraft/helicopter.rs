use core::convert::Into;

use crate::{
    absolute_position,
    aircraft::{Aircraft, AircraftState},
    data_from_gltf::load,
    input::ControlInputs,
};
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, Forces, Mass, Position, RigidBody, Rotation,
    WriteRigidBodyForces,
};
use bevy::{prelude::*, world_serialization::WorldInstanceReady};
use big_space::prelude::CellCoord;

pub fn mechanics(
    input: Res<ControlInputs>,
    state: ResMut<AircraftState>,
    _gizmos: Gizmos,
    aircraft: &mut Single<
        (
            &Position,
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
) {
    let (_, transform, force, _) = &mut **aircraft;
    if state.engine.on {
        let thrust_factor = 120_000.0;
        let thrust = transform.up() * thrust_factor * input.throttle;
        let torque = Vec3::new(input.pitch, input.yaw, input.roll);

        force.apply_force(thrust.into());
        force.apply_local_torque((torque * 500.0).into());
    }
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    level: Quat,
    object_pos: Vec3,
    object_cell: CellCoord,
    root_grid_id: Entity,
) {
    let path = "aircraft/helicopter/helicopter.gltf";
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
