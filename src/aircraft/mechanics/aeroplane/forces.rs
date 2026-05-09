use crate::aircraft::{
    BothSides, ControlSurfacesDeflection,
    mechanics::{AircraftPhysicsConfig, aeroplane::ASPECT_RATIO, alpha_deg, lift_coeff, rho},
};
use avian3d::prelude::{ReadRigidBodyForces, WriteRigidBodyForces, forces::ForcesItem};
use bevy::prelude::*;

pub fn steering(
    transform: &GlobalTransform,
    force: &mut ForcesItem,
    cs: &ControlSurfacesDeflection,
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
        y: cs.aileron.port * ROLL_FACTOR,
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
        y: cs.aileron.starboard * ROLL_FACTOR,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * roll_starboard_force * airspeed * factor,
        roll_starboard_point,
    );

    let pitch_force = Vec3 {
        x: 0.0,
        y: -cs.elevator * 50.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * pitch_force * airspeed * factor,
        pitch_point,
    );

    let yaw_force = Vec3 {
        x: cs.rudder * 50.0,
        y: 0.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * yaw_force * airspeed * factor,
        yaw_point,
    );
}

pub fn thrust(throttle: &f32, forward: &Dir3) -> Vec3 {
    let thrust_factor = 150_000.0;

    forward.as_vec3() * thrust_factor * throttle
}

pub fn induced_drag(lift_coeff: f32, rho: f32, speed: f32) -> f32 {
    let zero_lift_induced_drag_coeff = 0.0;
    let induced_drag_coeff =
        zero_lift_induced_drag_coeff + lift_coeff.powi(2) / std::f32::consts::PI * ASPECT_RATIO;
    let wingspan: f32 = 15.0;

    0.5 * rho * speed.powi(2) * induced_drag_coeff * wingspan.powi(2)
}

pub fn stabilise() -> Vec3 {
    Vec3::ZERO // TODO
}

pub fn lift(lift_coeff: f32, airspeed: f32, wing_area: f32, up: Dir3, rho: f32) -> Vec3 {
    let lift_force = lift_coeff * rho * (airspeed.powi(2) * 0.5) * wing_area;

    lift_force * up
}

pub fn canards_force(
    cs: &ControlSurfacesDeflection,
    force: &ForcesItem,
    transform: &GlobalTransform,
) -> BothSides<Vec3> {
    let velocity = BothSides {
        port: force.velocity_at_point(
            transform.translation() + transform.rotation() * Vec3::new(1.5, 0.6, -1.65),
        ),
        starboard: force.velocity_at_point(
            transform.translation() + transform.rotation() * Vec3::new(-1.5, 0.6, -1.65),
        ),
    };
    let canards = BothSides {
        port: Quat::from_rotation_x(cs.canards.port) * transform.rotation(),
        starboard: Quat::from_rotation_x(cs.canards.starboard) * transform.rotation(),
    };

    let canards_up = BothSides {
        port: Quat::from_rotation_x(cs.canards.port) * transform.up(),
        starboard: Quat::from_rotation_x(cs.canards.starboard) * transform.up(),
    };

    let alpha = BothSides {
        port: alpha_deg(
            &velocity.port,
            &GlobalTransform::from_rotation(canards.port),
        ),
        starboard: alpha_deg(
            &velocity.starboard,
            &GlobalTransform::from_rotation(canards.starboard),
        ),
    };

    const POTENTIAL_LIFT_FACTOR: f32 = 1.2;
    const VORTEX_LIFT_FACTOR: f32 = 0.5;

    let lift_coeff = BothSides {
        port: lift_coeff(alpha.port, POTENTIAL_LIFT_FACTOR, VORTEX_LIFT_FACTOR),
        starboard: lift_coeff(alpha.starboard, POTENTIAL_LIFT_FACTOR, VORTEX_LIFT_FACTOR),
    };
    let wing_area = 1.0;

    let velocity_dir = BothSides {
        port: velocity.port.normalize_or_zero(),
        starboard: velocity.starboard.normalize_or_zero(),
    };

    let speed: f32 = velocity.port.length();

    let airspeed = BothSides {
        port: transform.forward().dot(velocity_dir.port).clamp(0.0, 1.0) * speed,
        starboard: transform
            .forward()
            .dot(velocity_dir.starboard)
            .clamp(0.0, 1.0)
            * speed,
    };

    let all_other_stuff = BothSides {
        port: airspeed.port.powi(2)
            * 0.01
            * wing_area
            * rho(transform.translation().y.into()).density as f32,
        starboard: airspeed.starboard.powi(2)
            * 0.01
            * wing_area
            * rho(transform.translation().y.into()).density as f32,
    };

    let result = BothSides {
        port: canards_up.port * lift_coeff.port * all_other_stuff.port,
        starboard: canards_up.starboard * lift_coeff.starboard * all_other_stuff.starboard,
    };
    result
}
