// Thanks to Hermitao for making a prototype flight model (https://gist.github.com/Hermitao/0a908f8af19b11132e3bdb5ba4ef99f0)

use avian3d::prelude::{Forces, ReadRigidBodyForces, WriteRigidBodyForces, forces::ForcesItem};
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
                let thrust_factor = 85_000.;
                let thrust = transform.up() * thrust_factor * input.throttle;
                let torque = Vec3::new(input.pitch, input.yaw, input.roll);

                force.apply_force(thrust);
                force.apply_local_torque(torque * 500.0);
            }
        }
        AircraftTypes::Aeroplane => {
            if state.engine_on {
                steering(*transform, &mut *force, &*input);

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

                // smth wrong with this
                let lift_coeff = match aoa.to_radians() {
                    d if d < 0.3 => d / 15.0_f32.to_radians() * 5.0 + 0.5,
                    d if d < 0.5 => 1.2 * (1.0 - (d - 15.0) / 5.0),
                    _ => 0.02, // stalled
                };
                // dbg!(lift_coeff);

                let parasitic_drag = velocity.powf(2.0) + 0.8 * forward.cross(velocity_dir);
                // dbg!(parasitic_drag);

                let drag = (-velocity_dir * induced_drag(lift_coeff, rho, speed))
                    + (-velocity_dir * parasitic_drag);
                // dbg!(drag);
                force.apply_force(drag);

                // Stabilisation (idk)
                stabilise();

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

struct AircraftPhysicsConfig {
    pitch_point: Vec3,
    yaw_point: Vec3,
    roll_port_point: Vec3,
    roll_starboard_point: Vec3,
}

fn steering(transform: &GlobalTransform, force: &mut ForcesItem, input: &InputAxis) {
    let airspeed = transform.forward().dot(force.linear_velocity());
    let factor = 0.01;

    let physics_cfg = AircraftPhysicsConfig {
        pitch_point: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        },
        yaw_point: Vec3 {
            x: 0.0,
            y: 2.0,
            z: 7.0,
        },
        roll_port_point: Vec3 {
            x: -6.0,
            y: 0.0,
            z: 2.0,
        },
        roll_starboard_point: Vec3 {
            x: 6.0,
            y: 0.0,
            z: 2.0,
        },
    };

    let pitch_point = transform.translation() + transform.rotation() * physics_cfg.pitch_point;
    let yaw_point = transform.translation() + transform.rotation() * physics_cfg.yaw_point;

    let roll_port_point =
        transform.translation() + transform.rotation() * physics_cfg.roll_port_point;
    let roll_port_force = Vec3 {
        x: 0.0,
        y: -input.roll * 50.,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * roll_port_force * airspeed * factor,
        roll_port_point,
    );

    let roll_starboard_point =
        transform.translation() + transform.rotation() * physics_cfg.roll_starboard_point;
    let roll_starboard_force = Vec3 {
        x: 0.0,
        y: input.roll * 50.,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * roll_starboard_force * airspeed * factor,
        roll_starboard_point,
    );

    let pitch_force = Vec3 {
        x: 0.0,
        y: -input.pitch * 50.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * pitch_force * airspeed * factor,
        pitch_point,
    );

    let yaw_force = Vec3 {
        x: input.yaw * 50.0,
        y: 0.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * yaw_force * airspeed * factor,
        yaw_point,
    );
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
