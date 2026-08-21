use crate::aircraft::{Aircraft, AircraftState, BothSides, BothSidesF32, alpha_deg};
use avian3d::prelude::{Forces, Position, ReadRigidBodyForces};
use bevy::prelude::*;

pub fn fly_by_wire(
    input: &crate::input::ControlInputs,
    state: &mut AircraftState,
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
    let mut cs = state.control_surfaces;

    if let Some(fdm_cs) = &mut aircraft.3 {
        cs.elevator.port = -input.pitch;
        cs.elevator.starboard = -input.pitch;

        cs.aileron.port = -input.roll;
        cs.aileron.starboard = input.roll;

        // avian_fdm control inputs
        fdm_cs.aileron = -input.roll as f64;
        fdm_cs.elevator = input.pitch as f64;
        fdm_cs.rudder = -input.yaw as f64;
        fdm_cs.throttle = input.throttle as f64;

        cs.ground_brakes = input.ground_brakes;

        cs.aileron.port = cs.aileron.port.clamp(-1.0, 1.0);
        cs.aileron.starboard = cs.aileron.starboard.clamp(-1.0, 1.0);

        cs.elevator.port = cs.elevator.port.clamp(-1.0, 1.0);
        cs.elevator.starboard = cs.elevator.starboard.clamp(-1.0, 1.0);

        cs.rudder = input.yaw.clamp(-1.0, 1.0);

        let canards_angle = canards_angle(aircraft, *state);
        cs.canards = canards_angle;

        state.control_surfaces = cs;
    }
}

pub fn canards_angle(
    aircraft: &mut Single<
        (
            &Position,
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
    state: AircraftState,
) -> BothSides<f32> {
    let velocity = aircraft.2.linear_velocity(); // (TODO)
    let transform = aircraft.1;

    let alpha_deg = alpha_deg(&velocity, transform);

    // Canards pitch up while landing gear is deployed, maximising lift
    let offset = match state.landing_gear_deployed {
        false => 0.0,
        true => 20.0,
    };

    let canards_angle = if velocity.length() <= 30.0 {
        0.0
    } else {
        (offset - alpha_deg).clamp(-22.0, 50.0).to_radians() as f32
    };

    canards_angle.both_sides()
}
