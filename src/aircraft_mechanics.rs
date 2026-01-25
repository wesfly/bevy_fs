use crate::{Aircraft, AircraftState, InputAxis};
use avian3d::prelude::*;
use bevy::prelude::*;

// TODO put this into aircraft.rs
pub fn aircraft_mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut query: Query<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
    state: Res<AircraftState>,
) {
    let thrust_factor;

    if state.engine_on {
        thrust_factor = 64_000.
    } else {
        thrust_factor = 0.
    }

    let force = transform.up() * thrust_factor * (input.throttle);
    let torque = Vec3::new(input.pitch, input.yaw, input.roll);

    for mut forces in &mut query {
        forces.apply_force(force);
        forces.apply_local_torque(torque * 500.0);
    }
}
