// Thanks to Hermitao for making a prototype flight model (https://gist.github.com/Hermitao/0a908f8af19b11132e3bdb5ba4ef99f0)
mod fly_by_wire;
mod forces;
mod landing_gear;

use crate::aircraft::{AircraftState, ControlSurfacesDeflection};
use avian3d::prelude::{forces::ForcesItem, *};
use bevy::prelude::*;
use forces::*;

pub use fly_by_wire::fly_by_wire;

const ASPECT_RATIO: f32 = 1.0;

pub fn mechanics(
    mut state: &mut AircraftState,
    transform: &GlobalTransform,
    force: &mut ForcesItem,
    spatial_query: SpatialQuery,
    _gizmos: Gizmos,
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
        let alpha = super::alpha_deg(&velocity, transform);

        let canard_forces = canards_force(&control_surface, &force, transform);

        // port
        force.apply_force_at_point(
            canard_forces.port,
            transform.translation() + transform.rotation() * Vec3::new(1.5, 0.6, -1.65),
        );

        // starboard
        force.apply_force_at_point(
            canard_forces.starboard,
            transform.translation() + transform.rotation() * Vec3::new(-1.5, 0.6, -1.65),
        );

        // Polhamus Analogy Constants for the fighter jet
        let potential_lift_factor = 1.65;
        let vortex_lift_factor = 3.05;
        let lift_coeff = super::lift_coeff(alpha, potential_lift_factor, vortex_lift_factor);

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
        landing_gear::spring_forces(force, spatial_query, transform, time, &mut state);
    }
}
