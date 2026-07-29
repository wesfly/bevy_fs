use core::convert::Into;

use crate::{
    aircraft::{Aircraft, AircraftState},
    input::ControlInputs,
};
use avian3d::prelude::{Forces, Position, WriteRigidBodyForces};
use bevy::prelude::*;

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
