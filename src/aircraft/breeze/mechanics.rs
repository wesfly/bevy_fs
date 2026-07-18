use crate::aircraft::breeze::fly_by_wire::fly_by_wire;
use crate::{
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    input::ControlInputs,
};
use avian3d::dynamics::rigid_body::forces::WriteRigidBodyForces;
use avian3d::prelude::Forces;
use avian3d::prelude::Position;
use bevy::math::DVec3;
use bevy::prelude::*;

pub fn mechanics(
    input: Res<ControlInputs>,
    state: &mut ResMut<AircraftState>,
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
    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            todo!()
        }
        AircraftTypes::Breeze => {
            fly_by_wire(&input, state, aircraft);
            engine(&input, **state, aircraft);
        }
        AircraftTypes::J3Cub => {
            // TODO
            fly_by_wire(&input, state, aircraft);
        }
    }
}

fn engine(
    input: &Res<ControlInputs>,
    state: AircraftState,
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
    let (position, transform, forces, option_ctrl_inputs) = &mut **aircraft;
    forces.apply_force_at_point(
        DVec3::new(0.0, 10.0, 0.0) * 100.0 * input.throttle as f64,
        position.as_ivec3().as_dvec3() + transform.rotation.as_dquat() * DVec3::new(0.0, 10.0, 0.0),
    );
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
