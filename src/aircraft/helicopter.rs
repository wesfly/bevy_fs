use crate::{
    aircraft::{Aircraft, AircraftState},
    input::ControlInputs,
};
use avian3d::prelude::{Forces, WriteRigidBodyForces};
use bevy::prelude::*;

pub fn mechanics(
    input: Res<ControlInputs>,
    state: ResMut<AircraftState>,
    _gizmos: Gizmos,
    aircraft: &mut Single<
        (
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
) {
    let transform = &aircraft.0.clone();
    let force = &mut aircraft.1;
    if state.engine.on {
        let thrust_factor = 120_000.0;
        let thrust = transform.up() * thrust_factor * input.throttle;
        let torque = Vec3::new(input.pitch, input.yaw, input.roll);

        force.apply_force(thrust);
        force.apply_local_torque(torque * 500.0);
    }
}
