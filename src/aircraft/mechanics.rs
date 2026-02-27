// Thanks to Hermitao for making a prototype flight model (https://gist.github.com/Hermitao/0a908f8af19b11132e3bdb5ba4ef99f0)

use avian3d::prelude::{Forces, ReadRigidBodyForces, WriteRigidBodyForces};
use bevy::prelude::*;

use crate::{
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    input::InputAxis,
};

const ASPECT_RATIO: f32 = 1.0;

pub fn mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut force: Single<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
    state: Res<AircraftState>,
) {
    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            if state.engine_on {
                let thrust_factor = 64_000.;
                let thrust = transform.up() * thrust_factor * input.throttle;
                let torque = Vec3::new(input.pitch, input.yaw, input.roll);

                force.apply_force(thrust);
                force.apply_local_torque(torque * 500.0);
            }
        }
        AircraftTypes::Aeroplane => {
            if state.engine_on {
                let input_torque = Vec3::new(input.pitch, input.yaw, input.roll);
                force.apply_local_torque(input_torque * 500.5);

                let forward = transform.forward();

                let rho = rho();

                let velocity = force.linear_velocity();
                let velocity_dir = velocity.normalize_or_zero();
                let speed: f32 = velocity.length();

                force.apply_force(thrust(&input, &forward));

                // Angle of attack
                let sin = forward.cross(velocity_dir).dot(transform.right().as_vec3());
                let cos = forward.dot(velocity_dir);
                let aoa = -sin.atan2(cos).to_degrees();

                let lift_coeff = match aoa {
                    d if d < 15.0 => d / 15.0 * 1.0 + 0.5,
                    d if d < 20.0 => 1.2 * (1.0 - (d - 15.0) / 5.0),
                    _ => 0.2, // stalled
                };

                let parasitic_drag = velocity.powf(2.0) * 0.8 * forward.cross(velocity_dir);
                let drag = (-velocity_dir * induced_drag(lift_coeff, rho, speed))
                    + (-velocity_dir * parasitic_drag);
                force.apply_force(drag);

                // Stabilisation (idk)
                let stability_thingy = stabilise();
                force.apply_local_angular_acceleration(stability_thingy);

                // L = Cl * p * (v^2/2) * A
                // Lift = coefficient * density * (airspeed^2 / 2) * wing area
                let wing_area = 49.0;
                let airspeed = forward.dot(velocity_dir).clamp(0.0, 1.0) * speed;
                let lift = lift(lift_coeff, airspeed, wing_area, transform.up(), rho);
                force.apply_force(lift);
            }
        }
    }
}

fn rho() -> f32 {
    // TODO implement that whole air pressure stuff
    1.2041
}

fn thrust(input: &InputAxis, forward: &Dir3) -> Vec3 {
    let thrust_factor = 150_000.0;
    let thrust = forward.as_vec3() * thrust_factor * input.throttle;

    thrust
}

fn induced_drag(lift_coeff: f32, rho: f32, speed: f32) -> f32 {
    let zero_lift_induced_drag_coeff = 0.0;
    let induced_drag_coeff =
        zero_lift_induced_drag_coeff + lift_coeff.powi(2) / std::f32::consts::PI * ASPECT_RATIO;
    let wingspan: f32 = 15.0;
    let induced_drag = 0.5 * rho * speed.powi(2) * induced_drag_coeff * wingspan.powi(2);

    induced_drag
}

fn stabilise() -> Vec3 {
    Vec3::ZERO
}

fn lift(lift_coeff: f32, airspeed: f32, wing_area: f32, up: Dir3, rho: f32) -> Vec3 {
    let lift_force = lift_coeff * rho * (airspeed.powi(2) * 0.5) * wing_area;
    let lift_vector = lift_force * up;
    lift_vector
}
