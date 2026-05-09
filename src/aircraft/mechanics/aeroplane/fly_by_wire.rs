use crate::{
    aircraft::{Aircraft, AircraftState, BothSides, BothSidesExt, mechanics::alpha_deg},
    input::InputAxis,
};
use avian3d::prelude::{Forces, ReadRigidBodyForces};
use bevy::prelude::*;

pub fn fly_by_wire(
    input: &InputAxis,
    state: &mut AircraftState,
    aircraft: Single<(&GlobalTransform, Forces), With<Aircraft>>,
) {
    let mut cs = state.control_surfaces;
    cs.elevator = input.pitch;
    cs.ground_brakes = input.ground_brakes;
    cs.rudder = input.yaw;
    cs.canards = canards_angle(aircraft, *state);

    cs.aileron.port = -input.roll;
    cs.aileron.starboard = input.roll;

    state.control_surfaces = cs;

    state.engine.throttle = input.throttle;
}

pub fn canards_angle(
    aircraft: Single<(&GlobalTransform, Forces), With<Aircraft>>,
    state: AircraftState,
) -> BothSides<f32> {
    let velocity = aircraft.1.linear_velocity();
    let transform = aircraft.0;

    let alpha_deg = alpha_deg(&velocity, transform);

    // Canards work the other around way when landing gear is deployed, maximising lift
    let factor = match state.landing_gear_deployed {
        false => 1.0,
        true => -1.0,
    };

    let canards_angle = if velocity.length() <= 20.0 {
        0.0
    } else {
        (factor * alpha_deg).clamp(-22.0, 50.0).to_radians()
    };

    canards_angle.both_sides()
}
