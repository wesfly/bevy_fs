// Thanks to Hermitao for making a prototype flight model (https://gist.github.com/Hermitao/0a908f8af19b11132e3bdb5ba4ef99f0)
mod landing_gear;
use crate::{
    aircraft::{AircraftState, mechanics::AircraftPhysicsConfig},
    input::InputAxis,
};
use avian3d::prelude::{forces::ForcesItem, *};
use bevy::prelude::*;

const ASPECT_RATIO: f32 = 1.0;

pub fn mechanics(
    state: AircraftState,
    transform: &GlobalTransform,
    force: &mut ForcesItem,
    input: &InputAxis,
    spatial_query: SpatialQuery,
    gizmos: Gizmos,
    time: Res<Time>,
) {
    if state.engine_on {
        steering(transform, force, &input);

        let forward = transform.forward();

        let rho = super::rho(transform.translation().y as f64).density as f32;

        let velocity = force.linear_velocity();
        let velocity_dir = velocity.normalize_or_zero();
        let speed: f32 = velocity.length();

        force.apply_force(thrust(&input, &forward) * rho / 1.2041);

        // Angle of attack
        let alpha = super::alpha(&velocity, transform);

        let lift_coeff = super::lift_coeff(alpha);

        let parasitic_drag_coeff = match state.landing_gear_deployed {
            true => 2.2,
            false => 1.8,
        };
        let parasitic_drag =
            parasitic_drag_coeff * velocity.powf(2.0) + 0.5 * forward.cross(velocity_dir);

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

    if state.landing_gear_deployed && force.linear_velocity().length() <= 200.0 {
        landing_gear::spring_forces(force, spatial_query, transform, gizmos, time, state, input);
    }
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

    const ROLL_FACTOR: f32 = 20.0;

    let roll_port_point =
        transform.translation() + transform.rotation() * physics_cfg.roll_port_point;
    let roll_port_force = Vec3 {
        x: 0.0,
        y: -input.roll * ROLL_FACTOR,
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
        y: input.roll * ROLL_FACTOR,
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

fn thrust(input: &InputAxis, forward: &Dir3) -> Vec3 {
    let thrust_factor = 150_000.0;

    forward.as_vec3() * thrust_factor * input.throttle
}

fn induced_drag(lift_coeff: f32, rho: f32, speed: f32) -> f32 {
    let zero_lift_induced_drag_coeff = 0.0;
    let induced_drag_coeff =
        zero_lift_induced_drag_coeff + lift_coeff.powi(2) / std::f32::consts::PI * ASPECT_RATIO;
    let wingspan: f32 = 15.0;

    0.5 * rho * speed.powi(2) * induced_drag_coeff * wingspan.powi(2)
}

fn stabilise() -> Vec3 {
    Vec3::ZERO // TODO
}

fn lift(lift_coeff: f32, airspeed: f32, wing_area: f32, up: Dir3, rho: f32) -> Vec3 {
    let lift_force = lift_coeff * rho * (airspeed.powi(2) * 0.5) * wing_area;

    lift_force * up
}
