use crate::{
    aircraft::{Aircraft, AircraftState, BothSides, BothSidesExt, mechanics::alpha_deg},
    input::ControlInputs,
};
use avian3d::prelude::{Forces, ReadRigidBodyForces};
use bevy::prelude::*;

pub fn fly_by_wire(
    input: &ControlInputs,
    state: &mut AircraftState,
    aircraft: Single<(&GlobalTransform, Forces), With<Aircraft>>,
) {
    let mut cs = state.control_surfaces;

    let velocity = aircraft.1.linear_velocity();
    let transform = aircraft.0;

    let alpha_deg = alpha_deg(&velocity, transform);

    cs.elevator.port = -input.pitch;
    cs.elevator.starboard = -input.pitch;

    cs.aileron.port = -input.roll;
    cs.aileron.starboard = input.roll;

    // Flaps
    {
        let flap_factor: f32 = if state.landing_gear_deployed {
            0.07
        } else {
            0.05
        };

        // Fading in flaps smoothly
        let factor = ((alpha_deg - 5.0) * 5.0).clamp(0.0, 1.0);
        cs.aileron.port += alpha_deg * flap_factor * factor;
        cs.aileron.starboard += alpha_deg * flap_factor * factor;

        let elevator_factor = (alpha_deg * flap_factor * factor).clamp(0.0, 0.7);
        cs.elevator.port += elevator_factor;
        cs.elevator.starboard += elevator_factor;
    }

    cs.ground_brakes = input.ground_brakes;

    cs.aileron.port = cs.aileron.port.clamp(-1.0, 1.0);
    cs.aileron.starboard = cs.aileron.starboard.clamp(-1.0, 1.0);

    cs.elevator.port = cs.elevator.port.clamp(-1.0, 1.0);
    cs.elevator.starboard = cs.elevator.starboard.clamp(-1.0, 1.0);

    cs.rudder = input.yaw.clamp(-1.0, 1.0);

    cs.canards = canards_angle(aircraft, *state);

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

    // Canards pitch up while landing gear is deployed, maximising lift
    let factor = match state.landing_gear_deployed {
        false => 0.0,
        true => -20.0,
    };

    let canards_angle = if velocity.length() <= 20.0 {
        0.0
    } else {
        (factor + alpha_deg).clamp(-22.0, 50.0).to_radians()
    };

    canards_angle.both_sides()
}
