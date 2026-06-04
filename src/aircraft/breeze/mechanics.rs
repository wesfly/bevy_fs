use crate::aircraft::breeze::fly_by_wire::fly_by_wire;
use crate::{
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    input::ControlInputs,
};
use avian3d::prelude::Forces;
use bevy::prelude::*;

pub fn mechanics(
    input: Res<ControlInputs>,
    state: &mut ResMut<AircraftState>,
    aircraft: &mut Single<
        (
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
) {
    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            todo!()
        }
        AircraftTypes::Breeze => {
            fly_by_wire(&*input, state, aircraft);
        }
        AircraftTypes::J3Cub => {
            // TODO
            fly_by_wire(&*input, state, aircraft);
        }
    }
}

/*

// TODO (maybe useful for vortex lift)
fn lift_coeff(alpha_deg: f32, potential_lift_factor: f32, vortex_lift_factor: f32) -> f32 {
    let alpha = alpha_deg.to_radians();

    let sin_a = alpha.sin();
    let cos_a = alpha.cos();

    // Polhamus Equation
    let cl_potential = potential_lift_factor * sin_a * cos_a.powi(2);
    let cl_vortex = vortex_lift_factor * sin_a.powi(2) * cos_a;

    cl_potential + cl_vortex
}

*/
