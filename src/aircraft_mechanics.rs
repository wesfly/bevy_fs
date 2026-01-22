use crate::{Aircraft, AircraftState, InputAxis, Settings};
use avian3d::prelude::*;
use bevy::prelude::*;

pub fn aircraft_mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut query: Query<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
    settings: Res<Settings>,
    state: Res<AircraftState>,
) {
    // When controlling with buttons, the inertia is too high. This adjusts for that.
    let torque_factor;
    if settings.gamepad.enabled {
        torque_factor = 1200.
    } else {
        torque_factor = 3000.
    }

    let thrust_factor;

    if state.engine_on {
        thrust_factor = 84_000.
    } else {
        thrust_factor = 0.
    }

    let force = transform.up() * thrust_factor * (input.throttle);
    let torque = Vec3::new(input.pitch, input.yaw * 2.5, input.roll);

    for mut forces in &mut query {
        forces.apply_force(force);
        forces.apply_local_torque(torque * torque_factor);
    }
}
