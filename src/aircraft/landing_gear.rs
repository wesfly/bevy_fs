use crate::aircraft::AircraftState;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum LandingGearElements {
    FrontLdgGear,
    FrontLdgGearDoorFrontLeft,
    FrontLdgGearDoorFrontRight,
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
    // pub status: Option<LandingGearStatus>,
}

pub enum LandingGearCommands {
    Toggle,
    // Deploy,
    // Retract,
}

#[derive(Message)]
pub struct LandingGearCommand(pub LandingGearCommands);

#[derive(Resource, Deserialize, Debug)]
#[derive(Default)]
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


const LDG_GEAR_DEPLOY_SPD: f32 = 20.0_f32.to_radians();
const DOOR_DEPLOY_ANGLE: f32 = 80.0_f32.to_radians();

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
        if let Some(message) = landing_gear_messages.read().last() {
            match message.0 {
                LandingGearCommands::Toggle => {
                    *status = match *status {
                        LandingGearStatus::Deploying => LandingGearStatus::Retracting,
                        LandingGearStatus::Deployed => LandingGearStatus::Retracting,
                        LandingGearStatus::Retracting => LandingGearStatus::Deploying,
                        LandingGearStatus::Retracted => LandingGearStatus::Deploying,
                    };
                    *phase = LdgGearPhase::Phase1;
                    info!("Toggled ldg");
                }
            }
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
                        ready_vec.append(&mut vec![false, false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *phase = LdgGearPhase::Phase2;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFrontLeft => {
                            if transform.rotation.to_euler(EulerRot::XYZ).1 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_y(ldg_gear_speed * delta);
                            } else {
                                ready_vec[0] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorFrontRight => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).1 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_y(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[1] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftLeft => {
                            if transform.rotation.to_euler(EulerRot::XYZ).1 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_y(ldg_gear_speed * delta);
                            } else {
                                ready_vec[2] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftRight => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).1 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_y(-ldg_gear_speed * delta);
                            } else {
                                ready_vec[3] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {}
                        LandingGearElements::StarboardLdgGearDoorAft => {}

                        LandingGearElements::PortLdgGearDoorFront => {}
                        LandingGearElements::PortLdgGearDoorAft => {}
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
                        ready_vec.append(&mut vec![false, false]);
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
                        LandingGearElements::FrontLdgGearDoorFrontLeft => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).1 <= 0.0 {
                                transform.rotate_local_y(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorFrontRight => {
                            if transform.rotation.to_euler(EulerRot::XYZ).1 <= 0.0 {
                                transform.rotate_local_y(ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {}

                        LandingGearElements::PortLdgGearDoorFront => {}
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
                        ready_vec.append(&mut vec![false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *phase = LdgGearPhase::Phase2;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFrontLeft => {
                            if transform.rotation.to_euler(EulerRot::XYZ).1 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_y(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true;
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorFrontRight => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).1 <= DOOR_DEPLOY_ANGLE {
                                transform.rotate_local_y(ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true;
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {}
                        LandingGearElements::PortLdgGearDoorFront => {}
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
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= -2.0_f32.to_radians()
                            {
                                transform.rotate_local_x(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true
                            }
                        }

                        LandingGearElements::PortLdgGear => {
                            if transform.rotation.to_euler(EulerRot::XYZ).0 <= -2.0_f32.to_radians()
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
                        ready_vec.append(&mut vec![false, false, false, false]);
                    } else {
                        let all_true = &ready_vec.iter().all(|&x| x);
                        if *all_true {
                            *ready_vec = vec![];
                            *status = LandingGearStatus::Retracted;
                            break;
                        }
                    }
                    match element.ldg_gear_element {
                        LandingGearElements::FrontLdgGearDoorFrontLeft => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).1 <= 0.0 {
                                transform.rotate_local_y(ldg_gear_speed * delta)
                            } else {
                                ready_vec[0] = true
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorFrontRight => {
                            if transform.rotation.to_euler(EulerRot::XYZ).1 <= 0.0 {
                                transform.rotate_local_y(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[1] = true
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftLeft => {
                            if -transform.rotation.to_euler(EulerRot::XYZ).1 <= 0.0 {
                                transform.rotate_local_y(ldg_gear_speed * delta)
                            } else {
                                ready_vec[2] = true
                            }
                        }
                        LandingGearElements::FrontLdgGearDoorAftRight => {
                            if transform.rotation.to_euler(EulerRot::XYZ).1 <= 0.0 {
                                transform.rotate_local_y(-ldg_gear_speed * delta)
                            } else {
                                ready_vec[3] = true
                            }
                        }

                        LandingGearElements::StarboardLdgGearDoorFront => {}
                        LandingGearElements::StarboardLdgGearDoorAft => {}

                        LandingGearElements::PortLdgGearDoorFront => {}
                        LandingGearElements::PortLdgGearDoorAft => {}
                        _ => {}
                    }
                }
            }
        }
    }
}
