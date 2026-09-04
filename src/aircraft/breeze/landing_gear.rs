use crate::{
    aircraft::{Aircraft, AircraftState},
    bevy_to_aerospace_coords,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::ops::DerefMut;

#[derive(
    Debug,
    Deserialize,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Reflect,
    Component,
    Hash
)]
#[reflect(Component)]
#[type_path = "skein"]
pub enum LandingGearElement {
    FrontLdgGear,
    FrontLdgGearDoorFront,
    FrontLdgGearDoorAftLeft,
    FrontLdgGearDoorAftRight,

    StarboardLdgGear,
    StarboardLdgGearDoorFront,
    StarboardLdgGearDoorAft,

    PortLdgGear,
    PortLdgGearDoorFront,
    PortLdgGearDoorAft,
}

pub enum LandingGearCommands {
    Toggle,
}

#[derive(Message)]
pub struct LandingGearCommand(pub LandingGearCommands);

#[derive(Resource, Deserialize, Debug, Default, Clone)]
pub enum LandingGearStatus {
    Deploying,
    Deployed,
    Retracting,
    #[default]
    Retracted,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum LdgGearPhase {
    #[default]
    Phase1,
    Phase2,
    Phase3,
}

// Old transform, needs to be multiplied with bevy_to_aerospace_coords()
pub const LEFT_POS: Vec3 = Vec3::new(-1.3, -0.60, 2.1);
pub const RIGHT_POS: Vec3 = Vec3::new(1.3, -0.60, 2.1);
pub const NOSEWHEEL_POS: Vec3 = Vec3::new(0.0, -0.53, -2.9);

const LDG_GEAR_DEPLOY_SPD: f32 = 40.0_f32.to_radians();
const DOOR_DEPLOY_ANGLE: f32 = 80.0_f32.to_radians();
const WIDE_ANGLE: f32 = DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians();

#[derive(Clone, Copy)]
enum Axis {
    X,
    Z,
}

type GearTarget = (LandingGearElement, Axis, f32);

use LandingGearElement::*;

fn deploy_phase1_targets() -> [GearTarget; 7] {
    [
        (FrontLdgGearDoorFront, Axis::Z, -WIDE_ANGLE),
        (FrontLdgGearDoorAftLeft, Axis::Z, -DOOR_DEPLOY_ANGLE),
        (FrontLdgGearDoorAftRight, Axis::Z, DOOR_DEPLOY_ANGLE),
        (PortLdgGearDoorFront, Axis::Z, WIDE_ANGLE),
        (PortLdgGearDoorAft, Axis::X, -DOOR_DEPLOY_ANGLE),
        (StarboardLdgGearDoorFront, Axis::Z, -WIDE_ANGLE),
        (StarboardLdgGearDoorAft, Axis::X, -DOOR_DEPLOY_ANGLE),
    ]
}

fn deploy_phase2_targets() -> [GearTarget; 3] {
    [
        (FrontLdgGear, Axis::Z, -90.0_f32.to_radians()),
        (StarboardLdgGear, Axis::X, -210.0_f32.to_radians()),
        (PortLdgGear, Axis::X, -210.0_f32.to_radians()),
    ]
}

fn deploy_phase3_targets() -> [GearTarget; 3] {
    [
        (FrontLdgGearDoorFront, Axis::Z, 0.0),
        (PortLdgGearDoorFront, Axis::Z, 0.0),
        (StarboardLdgGearDoorFront, Axis::Z, 0.0),
    ]
}

fn retract_phase1_targets() -> [GearTarget; 3] {
    [
        (FrontLdgGearDoorFront, Axis::Z, -WIDE_ANGLE),
        (PortLdgGearDoorFront, Axis::Z, WIDE_ANGLE),
        (StarboardLdgGearDoorFront, Axis::Z, -WIDE_ANGLE),
    ]
}

fn retract_phase2_targets() -> [GearTarget; 3] {
    [
        (FrontLdgGear, Axis::Z, 0.0),
        (StarboardLdgGear, Axis::X, 0.0),
        (PortLdgGear, Axis::X, 0.0),
    ]
}

fn retract_phase3_targets() -> [GearTarget; 7] {
    [
        (FrontLdgGearDoorFront, Axis::Z, 0.0),
        (FrontLdgGearDoorAftLeft, Axis::Z, 0.0),
        (FrontLdgGearDoorAftRight, Axis::Z, 0.0),
        (PortLdgGearDoorFront, Axis::Z, 0.0),
        (StarboardLdgGearDoorFront, Axis::Z, 0.0),
        (PortLdgGearDoorAft, Axis::X, 0.0),
        (StarboardLdgGearDoorAft, Axis::X, 0.0),
    ]
}

fn step_axis_towards(
    current: &mut f32,
    transform: &mut Transform,
    axis: Axis,
    target: f32,
    max_step: f32,
) -> bool {
    let diff = target - *current;
    let reached = diff.abs() <= max_step;
    let step = if reached {
        diff
    } else {
        max_step.copysign(diff)
    };

    match axis {
        Axis::X => transform.rotate_local_x(step),
        Axis::Z => transform.rotate_local_z(step),
    }
    *current += step;
    reached
}

fn step_phase(
    query: &mut Query<(&mut Transform, &LandingGearElement)>,
    angles: &mut std::collections::HashMap<LandingGearElement, f32>,
    targets: &[GearTarget],
    max_step: f32,
) -> bool {
    let mut all_reached = true;
    for (mut transform, element) in query.iter_mut() {
        let Some(&(_, axis, target)) = targets.iter().find(|(e, ..)| e == element) else {
            continue;
        };
        let current = angles.entry(*element).or_insert(0.0);
        if !step_axis_towards(current, &mut transform, axis, target, max_step) {
            all_reached = false;
        }
    }
    all_reached
}

#[derive(Component)]
pub struct LandingGear;
impl LandingGear {
    pub fn operate_landing_gear(
        mut landing_gear_messages: MessageReader<LandingGearCommand>,
        mut query: Query<(&mut Transform, &LandingGearElement)>,
        mut state: ResMut<AircraftState>,
        mut status: ResMut<LandingGearStatus>,
        time: Res<Time>,
        mut phase: Local<LdgGearPhase>,
        mut hash_ldg: Local<std::collections::HashMap<LandingGearElement, f32>>,
    ) {
        if landing_gear_messages.read().last().is_some() {
            *phase = LdgGearPhase::Phase1;
            match *status {
                LandingGearStatus::Deployed => {
                    *status = LandingGearStatus::Retracting;
                }
                LandingGearStatus::Retracted => {
                    *status = LandingGearStatus::Deploying;
                }
                _ => {}
            };
        }

        match *status {
            LandingGearStatus::Deploying => {
                Self::deploy_landing_gear(
                    &mut query,
                    &mut state,
                    &mut status,
                    &time,
                    &mut phase,
                    &mut hash_ldg,
                );
            }
            LandingGearStatus::Retracting => {
                Self::retract_landing_gear(
                    &mut query,
                    &mut status,
                    &time,
                    &mut phase,
                    &mut hash_ldg,
                );
                state.landing_gear_deployed = false;
            }
            _ => {}
        }
    }

    pub fn deploy_landing_gear(
        query: &mut Query<(&mut Transform, &LandingGearElement)>,
        state: &mut AircraftState,
        status: &mut LandingGearStatus,
        time: &Time,
        phase: &mut LdgGearPhase,
        mut hash_ldg: &mut std::collections::HashMap<LandingGearElement, f32>,
    ) {
        let max_step = LDG_GEAR_DEPLOY_SPD * time.delta_secs();

        let phase_done = match *phase {
            LdgGearPhase::Phase1 => {
                step_phase(query, &mut hash_ldg, &deploy_phase1_targets(), max_step)
            }
            LdgGearPhase::Phase2 => {
                step_phase(query, &mut hash_ldg, &deploy_phase2_targets(), max_step)
            }
            LdgGearPhase::Phase3 => {
                step_phase(query, &mut hash_ldg, &deploy_phase3_targets(), max_step)
            }
        };

        if !phase_done {
            return;
        }

        match *phase {
            LdgGearPhase::Phase1 => *phase = LdgGearPhase::Phase2,
            LdgGearPhase::Phase2 => *phase = LdgGearPhase::Phase3,
            LdgGearPhase::Phase3 => {
                *status = LandingGearStatus::Deployed;
                state.landing_gear_deployed = true;
            }
        }
    }

    pub fn retract_landing_gear(
        query: &mut Query<(&mut Transform, &LandingGearElement)>,
        status: &mut LandingGearStatus,
        time: &Time,
        phase: &mut LdgGearPhase,
        mut hash_ldg: &mut std::collections::HashMap<LandingGearElement, f32>,
    ) {
        let max_step = LDG_GEAR_DEPLOY_SPD * time.delta_secs();

        let phase_done = match *phase {
            LdgGearPhase::Phase1 => {
                step_phase(query, &mut hash_ldg, &retract_phase1_targets(), max_step)
            }
            LdgGearPhase::Phase2 => {
                step_phase(query, &mut hash_ldg, &retract_phase2_targets(), max_step)
            }
            LdgGearPhase::Phase3 => {
                step_phase(query, &mut hash_ldg, &retract_phase3_targets(), max_step)
            }
        };

        if !phase_done {
            return;
        }

        match *phase {
            LdgGearPhase::Phase1 => *phase = LdgGearPhase::Phase2,
            LdgGearPhase::Phase2 => *phase = LdgGearPhase::Phase3,
            LdgGearPhase::Phase3 => *status = LandingGearStatus::Retracted,
        }
    }
}
const REST: f64 = 1.2;

const MAIN_STRENGTH: f64 = 400_000.0;
const MAIN_DAMPING: f64 = 59_000.0;

const NOSE_STRENGTH: f64 = 158_000.0;
const NOSE_DAMPING: f64 = 17_500.0;

const MAX_FORCE: f64 = 2_000_000.0; // safety clamp only

const MU_LATERAL: f64 = 0.8; // tire lateral grip coefficient
const MU_BRAKE: f64 = 0.6; // dry max braking friction coefficient
const TIRE_RELAXATION_TIME: f64 = 0.1; // s, how fast lateral slip is killed
const G: f64 = 9.81;

pub fn spring_forces(
    spatial_query: SpatialQuery,
    mut query: Single<
        (
            &Position,
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
    mut state: ResMut<AircraftState>,
) {
    let (position, transform, force, _) = query.deref_mut();

    if !(state.landing_gear_deployed && force.linear_velocity().length() <= 200.0) {
        return;
    }

    let landing_gear = [
        bevy_to_aerospace_coords() * LEFT_POS,
        bevy_to_aerospace_coords() * RIGHT_POS,
        bevy_to_aerospace_coords() * NOSEWHEEL_POS,
    ];

    let mut on_ground_vec = [false; 3];

    for (i, gear_pos) in landing_gear.iter().enumerate() {
        let is_nosewheel = i == 2;
        let (strength, damping) = if is_nosewheel {
            (NOSE_STRENGTH, NOSE_DAMPING)
        } else {
            (MAIN_STRENGTH, MAIN_DAMPING)
        };
        let rest = if is_nosewheel { REST - 0.1 } else { REST };

        let filter = SpatialQueryFilter::DEFAULT;
        let origin = position.0 + (transform.rotation * gear_pos).as_dvec3();
        let ray_dir = transform.local_z();

        let Some(hit) = spatial_query.cast_ray(origin, ray_dir, rest, true, &filter) else {
            continue;
        };

        let distance = hit.distance.max(1e-4);
        let spring_dir = -transform.local_z();
        on_ground_vec[i] = true;

        let contact_point = origin + ray_dir.as_dvec3() * distance;

        //============================== springs ==============================
        let spring_vel = spring_dir.dot(force.velocity_at_point(contact_point).as_vec3()) as f64;
        let spring_scalar = spring(distance, rest, strength, damping, spring_vel);
        let spring_force = (spring_scalar * spring_dir.as_dvec3()).clamp_length_max(MAX_FORCE);

        force.apply_force_at_point(spring_force, contact_point);

        let normal_load = spring_scalar.max(0.0);

        //============================== steering/anti-drift ==============================
        let steering_dir = if is_nosewheel {
            Quat::from_rotation_y(20.0 * state.control_surfaces.rudder.to_radians())
                * transform.local_y()
        } else {
            transform.local_y()
        };
        let vel_at_contact_point = force.velocity_at_point(contact_point);
        let steering_vel = steering_dir.dot(vel_at_contact_point.as_vec3()) as f64;

        let effective_mass = normal_load / G;
        let desired_lateral_accel = -steering_vel / TIRE_RELAXATION_TIME;
        let max_lateral_force = normal_load * MU_LATERAL;
        let lateral_force = (effective_mass * desired_lateral_accel * steering_dir.as_dvec3())
            .clamp_length_max(max_lateral_force);
        force.apply_force_at_point(lateral_force, contact_point);

        //============================== brakes ==============================
        if !is_nosewheel {
            let tire_speed = transform.local_x().dot(vel_at_contact_point.as_vec3()) as f64;
            let brake_input = if state.parking_brake {
                1.0
            } else {
                state.control_surfaces.ground_brakes as f64
            };

            let max_brake_force = normal_load * MU_BRAKE * brake_input;
            let braking_force =
                tire_speed.signum() * max_brake_force * -transform.local_x().as_dvec3();

            force.apply_force_at_point(braking_force, contact_point);
        }
    }

    let ldg_gear_on_ground = on_ground_vec.iter().filter(|&&x| x).count();
    state.on_ground = ldg_gear_on_ground >= 2;
}

fn spring(
    distance: f64,
    rest_length: f64,
    strength: f64,
    damping_factor: f64,
    velocity: f64,
) -> f64 {
    let offset = rest_length - distance;
    if offset >= 0.0 {
        let spring = offset * strength;
        let damping = velocity * damping_factor;
        return spring - damping;
    }
    0.0
}
