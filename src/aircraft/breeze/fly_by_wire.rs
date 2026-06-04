use crate::aircraft::{Aircraft, AircraftState, BothSides, BothSidesExt, alpha_deg};
use avian3d::prelude::{Forces, ReadRigidBodyForces};
use bevy::prelude::*;

pub fn fly_by_wire(
    input: &crate::input::ControlInputs,
    state: &mut AircraftState,
    aircraft: &mut Single<
        (
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
) {
    let mut cs = state.control_surfaces;

    if let Some(fdm_cs) = &mut aircraft.2 {
        cs.elevator.port = -input.pitch;
        cs.elevator.starboard = -input.pitch;

        cs.aileron.port = -input.roll;
        cs.aileron.starboard = input.roll;

        // avian_fdm control inputs
        fdm_cs.aileron = -input.roll;
        fdm_cs.elevator = input.pitch;
        fdm_cs.throttle = input.throttle;
        fdm_cs.rudder = -input.yaw;

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
}

pub fn canards_angle(
    aircraft: &mut Single<
        (
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
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
        ((factor + alpha_deg).clamp(-22.0, 50.0) as f32).to_radians()
    };

    canards_angle.both_sides()
}
