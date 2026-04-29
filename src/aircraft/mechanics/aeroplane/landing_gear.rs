use crate::{
    aircraft::{
        AircraftState,
        landing_gear::{LEFT_POS, NOSEWHEEL_POS, RIGHT_POS},
    },
    input::InputAxis,
};
use avian3d::prelude::{forces::ForcesItem, *};
use bevy::prelude::*;

const REST: f32 = 1.2;
const STRENGTH: f32 = 200_000.0;
const DAMPING: f32 = 5_000.0;

const MAX_FORCE: f32 = 1_000_000.0;
const MAX_BRAKING_FORCE: f32 = 100_000.0;

// Making Custom Car Physics in Unity (for Very Very Valet)
// https://www.youtube.com/watch?v=CdPYlj5uZeI
pub fn spring_forces(
    force: &mut ForcesItem,
    spatial_query: SpatialQuery,
    transform: &GlobalTransform,
    mut _gizmos: Gizmos,
    time: Res<Time>,
    state: AircraftState,
    input: &InputAxis,
) {
    let landing_gear = vec![LEFT_POS, RIGHT_POS, NOSEWHEEL_POS];

    for (i, gear_pos) in landing_gear.iter().enumerate() {
        let is_nosewheel = i == 2;
        let strength = if is_nosewheel {
            STRENGTH * 0.8
        } else {
            STRENGTH
        };
        let rest = if is_nosewheel { REST + 0.1 } else { REST };

        let filter = SpatialQueryFilter::DEFAULT;
        let origin = transform.translation() + transform.rotation() * gear_pos;
        let ray_dir = transform.down();

        if let Some(hit) = spatial_query.cast_ray(origin, ray_dir, rest, true, &filter) {
            let spring_dir = transform.up();
            if hit.distance != 0.0 {
                // The point where the gear touches the ground
                let contact_point = origin + ray_dir * hit.distance;

                //============================== springs ==============================
                let spring_vel = spring_dir.dot(force.velocity_at_point(contact_point));

                let spring_force = (spring(hit.distance, rest, strength, DAMPING, spring_vel)
                    * spring_dir)
                    .clamp_length_max(MAX_FORCE);

                // This is applied three times because three rays are cast
                force.apply_force_at_point(spring_force, origin);

                //============================== steering/anti-drift ==============================
                let steering_dir = if is_nosewheel {
                    Quat::from_rotation_y(20.0 * input.yaw.to_radians()) * transform.right()
                } else {
                    transform.right()
                };
                let vel_at_contact_point = force.velocity_at_point(contact_point);

                let steering_vel = steering_dir.dot(vel_at_contact_point);

                let tire_grip_factor = 0.6;
                let desired_vel_change = -steering_vel * tire_grip_factor;

                let desired_accel = desired_vel_change * time.delta_secs();
                let tire_mass = 3_300.0; // The mass that rests on each tire
                force.apply_force_at_point(steering_dir * tire_mass * desired_accel, contact_point);

                //============================== brakes ==============================

                if !is_nosewheel {
                    let tire_speed = transform.forward().dot(vel_at_contact_point);
                    let braking_input = if state.parking_brake {
                        0.9
                    } else {
                        input.ground_brakes * 0.6
                    };
                    let braking_coeff = 20.0;
                    let braking_force = (braking_input
                        * tire_speed.signum()
                        * tire_mass
                        * tire_grip_factor
                        * braking_coeff
                        * transform.back())
                    .clamp_length_max(MAX_BRAKING_FORCE);

                    force.apply_force_at_point(braking_force, contact_point);
                }
            }
        }
    }
}

fn spring(
    distance: f32,
    rest_length: f32,
    strength: f32,
    damping_factor: f32,
    velocity: f32,
) -> f32 {
    let offset = rest_length - distance;

    if offset >= 0.0 {
        let spring = offset * strength;

        let damping = velocity * damping_factor;

        return spring - damping;
    }

    0.0
}
