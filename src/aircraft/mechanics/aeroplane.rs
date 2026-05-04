// Thanks to Hermitao for making a prototype flight model (https://gist.github.com/Hermitao/0a908f8af19b11132e3bdb5ba4ef99f0)
mod landing_gear;
use crate::{
    aircraft::{
        Aircraft, AircraftState, ControlSurfacesDeflection,
        mechanics::{AircraftPhysicsConfig, canards_angle},
    },
    input::InputAxis,
};
use avian3d::prelude::{forces::ForcesItem, *};
use bevy::prelude::*;

const ASPECT_RATIO: f32 = 1.0;

pub fn fly_by_wire(
    input: &InputAxis,
    state: &mut AircraftState,
    aircraft: Single<(&GlobalTransform, Forces), With<Aircraft>>,
) {
    let mut cs = state.control_surfaces;
    cs.elevator = input.pitch;
    cs.aileron = input.roll;
    cs.ground_brakes = input.ground_brakes;
    cs.rudder = input.yaw;
    cs.canards = canards_angle(aircraft, *state).0;

    state.control_surfaces = cs;

    state.engine.throttle = input.throttle;
}

pub fn mechanics(
    mut state: &mut AircraftState,
    transform: &GlobalTransform,
    force: &mut ForcesItem,
    spatial_query: SpatialQuery,
    gizmos: Gizmos,
    time: Res<Time>,
) {
    let velocity = force.linear_velocity();
    let velocity_dir = velocity.normalize_or_zero();
    let speed: f32 = velocity.length();

    let control_surface: ControlSurfacesDeflection = state.control_surfaces;

    if state.engine.on {
        steering(transform, force, &control_surface);

        let forward = transform.forward();

        let rho = super::rho(transform.translation().y as f64).density as f32;

        force.apply_force(thrust(&state.engine.throttle, &forward) * rho / 1.2041);

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
        if control_surface.elevator > 0.0 {
            let factor = if state.on_ground { 1000.0 } else { 5.0 };
            force.apply_force_at_point(
                transform.up() * speed * factor * control_surface.elevator,
                transform.translation() + transform.rotation() * Vec3::new(0.0, 0.6, -1.8),
            );
            // gizmos.arrow(
            //     transform.translation() + transform.rotation() * Vec3::new(0.0, 0.6, -1.8),
            //     transform.translation()
            //         + transform.rotation() * Vec3::new(0.0, 0.6, -1.8)
            //         + transform.up() * speed * 1.0 * input.pitch,
            //     Color::BLACK,
            // );
        }

        landing_gear::spring_forces(force, spatial_query, transform, gizmos, time, &mut state);
    }
}

fn steering(
    transform: &GlobalTransform,
    force: &mut ForcesItem,
    input: &ControlSurfacesDeflection,
) {
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
        y: -input.aileron * ROLL_FACTOR,
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
        y: input.aileron * ROLL_FACTOR,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * roll_starboard_force * airspeed * factor,
        roll_starboard_point,
    );

    let pitch_force = Vec3 {
        x: 0.0,
        y: -input.elevator * 50.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * pitch_force * airspeed * factor,
        pitch_point,
    );

    let yaw_force = Vec3 {
        x: input.rudder * 50.0,
        y: 0.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * yaw_force * airspeed * factor,
        yaw_point,
    );
}

fn thrust(throttle: &f32, forward: &Dir3) -> Vec3 {
    let thrust_factor = 150_000.0;

    forward.as_vec3() * thrust_factor * throttle
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
