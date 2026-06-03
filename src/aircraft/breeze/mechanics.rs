use crate::aircraft::breeze::fly_by_wire::fly_by_wire;
use crate::{
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    input::ControlInputs,
};
use avian3d::prelude::{Forces, WriteRigidBodyForces};
use bevy::prelude::*;

pub fn mechanics(
    input: Res<ControlInputs>,
    mut state: ResMut<AircraftState>,
    _gizmos: Gizmos,
    mut aircraft: Single<
        (&Transform, Forces, &mut avian_fdm::prelude::ControlInputs),
        With<Aircraft>,
    >,
) {
    let transform = &aircraft.0.clone();
    let force = &mut aircraft.1;
    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            if state.engine.on {
                let thrust_factor = 120_000.;
                let thrust = transform.up() * thrust_factor * input.throttle;
                let torque = Vec3::new(input.pitch, input.yaw, input.roll);

                force.apply_force(thrust);
                force.apply_local_torque(torque * 500.0);
            }
        }
        AircraftTypes::Breeze => {
            fly_by_wire(&*input, &mut state, aircraft);
        }
        AircraftTypes::J3Cub => {
            fly_by_wire(&*input, &mut state, aircraft);
        }
    }
}

#[allow(unused)] // TODO
fn lift_coeff(alpha_deg: f32, potential_lift_factor: f32, vortex_lift_factor: f32) -> f32 {
    let alpha = alpha_deg.to_radians();

    let sin_a = alpha.sin();
    let cos_a = alpha.cos();

    // Polhamus Equation
    let cl_potential = potential_lift_factor * sin_a * cos_a.powi(2);
    let cl_vortex = vortex_lift_factor * sin_a.powi(2) * cos_a;

    cl_potential + cl_vortex
}
