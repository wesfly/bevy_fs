use crate::{
    aircraft::{Aircraft, AircraftState},
    bevy_to_aerospace_coords,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use serde::Deserialize;
use std::ops::DerefMut;

#[derive(Debug, Deserialize)]
pub enum LandingGearElements {
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

#[derive(Debug, Deserialize, Component)]
pub struct LandingGearElement {
    pub ldg_gear_element: LandingGearElements,
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

#[derive(Default)]
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

#[derive(Component)]
pub struct LandingGear;
impl LandingGear {
    pub fn operate_landing_gear(
        mut landing_gear_messages: MessageReader<LandingGearCommand>,
        query: Query<(&mut Transform, &LandingGearElement)>,
        mut state: ResMut<AircraftState>,
        mut status: ResMut<LandingGearStatus>,
        time: Res<Time>,
        mut phase: Local<LdgGearPhase>,
        ready_vec: Local<Vec<bool>>,
    ) {
        if landing_gear_messages.read().last().is_some() {
            match *status {
                LandingGearStatus::Deployed => {
                    *phase = LdgGearPhase::Phase1;
                    *status = LandingGearStatus::Retracting;
                    info!("Toggled landing gear");
                }
                LandingGearStatus::Retracted => {
                    *phase = LdgGearPhase::Phase1;
                    *status = LandingGearStatus::Deploying;
                    info!("Toggled landing gear");
                }
                _ => {}
            };
        }

        match *status {
            LandingGearStatus::Deploying => {
                Self::deploy_landing_gear(query, state, status, time, phase, ready_vec)
            }
            LandingGearStatus::Retracting => {
                Self::retract_landing_gear(query, status, time, phase, ready_vec);
                state.landing_gear_deployed = false
            }

            _ => {}
        }
    }

    // TODO: there is a one-frame error because the gear elements are only stopped
    // when it's above a certain threshold, but it doesn't snap back to where it should be

    pub fn deploy_landing_gear(
        query: Query<(&mut Transform, &LandingGearElement)>,
        mut state: ResMut<AircraftState>,
        mut status: ResMut<LandingGearStatus>,
        time: Res<Time>,
        mut phase: Local<LdgGearPhase>,
        mut ready_vec: Local<Vec<bool>>,
    ) {
        let delta = time.delta_secs();

        let ldg_gear_speed = match *status {
            LandingGearStatus::Deploying => LDG_GEAR_DEPLOY_SPD,
            LandingGearStatus::Deployed => 0.0,
            LandingGearStatus::Retracting => -LDG_GEAR_DEPLOY_SPD,
            LandingGearStatus::Retracted => 0.0,
        };

        for (mut transform, element) in query {
            match *phase {
                LdgGearPhase::Phase1 => {
                    if ready_vec.is_empty() {
                        ready_vec
                            .append(&mut vec![false, false, false, false, false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *phase = LdgGearPhase::Phase2;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFront => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2
                                <= DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians()
                            {
                                transform.rotate_local_z(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[0] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftLeft => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_z(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[1] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftRight => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_z(ldg_gear_speed * delta);
                            } else {
                                ready_vec[2] = true;
                            }
                        }

                        LandingGearElements::PortLdgGearDoorFront => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2
                                <= DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians()
                            {
                                transform.rotate_local_z(ldg_gear_speed * delta);
                            } else {
                                ready_vec[3] = true;
                            }
                        }
                        LandingGearElements::PortLdgGearDoorAft => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).0 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_x(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[4] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2
                                <= DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians()
                            {
                                transform.rotate_local_z(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[5] = true;
                            }
                        }
                        LandingGearElements::StarboardLdgGearDoorAft => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).0 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_x(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[6] = true;
                            }
                        }

                        _ => {}
                    }
                }

                LdgGearPhase::Phase2 => {
                    if ready_vec.is_empty() {
                        ready_vec.append(&mut vec![false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *phase = LdgGearPhase::Phase3;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGear => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).0
                                <= 90.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGear => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).0
                                <= 110.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true;
                            }
                        }

                        LandingGearElements::PortLdgGear => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).0
                                <= 110.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[2] = true;
                            }
                        }
                        _ => {}
                    }
                }
                LdgGearPhase::Phase3 => {
                    if ready_vec.is_empty() {
                        ready_vec.append(&mut vec![false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *status = LandingGearStatus::Deployed;
                            state.landing_gear_deployed = true;
                            break;
                        }
                    }

                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFront => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true;
                            }
                        }

                        LandingGearElements::PortLdgGearDoorFront => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[1] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(ldg_gear_speed * delta);
                            } else {
                                ready_vec[2] = true;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub fn retract_landing_gear(
        query: Query<(&mut Transform, &LandingGearElement)>,
        mut status: ResMut<LandingGearStatus>,
        time: Res<Time>,
        mut phase: Local<LdgGearPhase>,
        mut ready_vec: Local<Vec<bool>>,
    ) {
        let delta = time.delta_secs();

        let ldg_gear_speed = match *status {
            LandingGearStatus::Deploying => LDG_GEAR_DEPLOY_SPD,
            LandingGearStatus::Deployed => 0.0,
            LandingGearStatus::Retracting => -LDG_GEAR_DEPLOY_SPD,
            LandingGearStatus::Retracted => 0.0,
        };

        for (mut transform, element) in query {
            match *phase {
                LdgGearPhase::Phase1 => {
                    if ready_vec.is_empty() {
                        ready_vec.append(&mut vec![false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *phase = LdgGearPhase::Phase2;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFront => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2
                                <= DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians()
                            {
                                transform.rotate_local_z(ldg_gear_speed * delta);
                            } else {
                                ready_vec[0] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2
                                <= DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians()
                            {
                                transform.rotate_local_z(ldg_gear_speed * delta);
                            } else {
                                ready_vec[1] = true;
                            }
                        }
                        LandingGearElements::PortLdgGearDoorFront => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2
                                <= DOOR_DEPLOY_ANGLE + 20.0_f32.to_radians()
                            {
                                transform.rotate_local_z(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[2] = true;
                            }
                        }
                        _ => {}
                    }
                }

                LdgGearPhase::Phase2 => {
                    if ready_vec.is_empty() {
                        ready_vec.append(&mut vec![false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *phase = LdgGearPhase::Phase3;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGear => {
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= 10.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true
                            }
                        }

                        LandingGearElements::StarboardLdgGear => {
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= 10.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true
                            }
                        }

                        LandingGearElements::PortLdgGear => {
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= 10.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[2] = true
                            }
                        }
                        _ => {}
                    }
                }

                LdgGearPhase::Phase3 => {
                    if ready_vec.is_empty() {
                        ready_vec
                            .append(&mut vec![false, false, false, false, false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *status = LandingGearStatus::Retracted;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFront => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftLeft => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftRight => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(ldg_gear_speed * delta)
                            } else {
                                ready_vec[2] = true
                            }
                        }

                        LandingGearElements::PortLdgGearDoorFront => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(ldg_gear_speed * delta);
                            } else {
                                ready_vec[3] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {
                            if transform.rotation.to_euler(EulerRot::XYZ).2 <= 0.0 {
                                transform.rotate_local_z(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[4] = true;
                            }
                        }

                        LandingGearElements::PortLdgGearDoorAft => {
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= 0.0 {
                                transform.rotate_local_x(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[5] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorAft => {
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= 0.0 {
                                transform.rotate_local_x(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[6] = true;
                            }
                        }

                        _ => {}
                    }
                }
            }
        }
    }
}

const REST: f32 = 1.2;
const STRENGTH: f32 = 200_000.0;
const DAMPING: f32 = 15_000.0;

const MAX_FORCE: f32 = 1_000_000.0;
const MAX_BRAKING_FORCE: f32 = 100_000.0;

// Making Custom Car Physics in Unity (for Very Very Valet)
// https://www.youtube.com/watch?v=CdPYlj5uZeI
pub fn spring_forces(
    spatial_query: SpatialQuery,
    mut query: Single<
        (
            &Transform,
            Forces,
            Option<&mut avian_fdm::prelude::ControlInputs>,
        ),
        With<Aircraft>,
    >,
    time: Res<Time>,
    mut state: ResMut<AircraftState>,
) {
    let (transform, force, _) = query.deref_mut();

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

        let rest = if is_nosewheel { REST - 0.1 } else { REST };

        let filter = SpatialQueryFilter::DEFAULT;
        let origin = transform.translation + transform.rotation * gear_pos;
        let ray_dir = transform.local_z();

        if let Some(hit) = spatial_query.cast_ray(origin, ray_dir, rest, true, &filter) {
            if hit.distance == 0.0 {
                // warn!("Landing gear hit.distance = 0, skipping");
                continue;
            }

            let spring_dir = -transform.local_z();

            on_ground_vec[i] = true;

            // The point where the gear touches the ground
            let contact_point = origin + ray_dir * hit.distance;

            //============================== springs ==============================
            let spring_vel = spring_dir.dot(force.velocity_at_point(contact_point));

            let spring_force = (spring(hit.distance, rest, STRENGTH, DAMPING, spring_vel)
                * spring_dir)
                .clamp_length_max(MAX_FORCE);

            // This is applied three times because three rays are cast
            force.apply_force_at_point(spring_force, origin);

            //============================== steering/anti-drift ==============================
            let steering_dir = if is_nosewheel {
                Quat::from_rotation_y(20.0 * state.control_surfaces.rudder.to_radians())
                    * transform.local_y()
            } else {
                transform.local_y()
            };
            let vel_at_contact_point = force.velocity_at_point(contact_point);

            let steering_vel = steering_dir.dot(vel_at_contact_point);

            let tire_grip_factor = 0.5;
            let desired_vel_change = -steering_vel * tire_grip_factor;

            let desired_accel = desired_vel_change / time.delta_secs();
            let tire_mass = 3_300.0 / hit.distance; // The mass that rests on each tire
            force.apply_force_at_point(
                (steering_dir * tire_mass * desired_accel * 10.0).clamp_length_max(10000.0),
                contact_point,
            );

            //============================== brakes ==============================

            if !is_nosewheel {
                let tire_speed = transform.local_x().dot(vel_at_contact_point);
                let braking_input = if state.parking_brake {
                    0.9
                } else {
                    state.control_surfaces.ground_brakes * 0.6
                };
                let braking_coeff = 50.0;
                let braking_force = (braking_input
                    * tire_speed.signum()
                    * tire_mass
                    * tire_grip_factor
                    * braking_coeff
                    * -transform.local_x())
                .clamp_length_max(MAX_BRAKING_FORCE);

                force.apply_force_at_point(braking_force, contact_point);
            }
        }
    }

    let ldg_gear_on_ground = on_ground_vec.iter().filter(|&&x| x).count();
    state.on_ground = ldg_gear_on_ground >= 2;
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
