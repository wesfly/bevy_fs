use crate::aircraft::{Aircraft, AircraftState, ControlSurfaces, RotorTypes};
use crate::input::InputAxis;
use avian3d::prelude::Forces;
use bevy::prelude::*;

pub fn update_rotors(
    query: Query<(&mut Transform, &RotorTypes)>,
    state: Res<AircraftState>,
    time: Res<Time>,
) {
    if state.engine.on {
        for (mut rotor, rotor_type) in query {
            match rotor_type {
                RotorTypes::Main => rotor.rotate_local_y(100.0 * time.delta_secs()),
                RotorTypes::Rear => rotor.rotate_local_z(100.0 * time.delta_secs()),
            }
        }
    }
}

pub fn update_control_surfaces(
    ctrl_surfaces: Query<(&mut Transform, &ControlSurfaces), Without<Aircraft>>,
    aircraft: Single<(&GlobalTransform, Forces), With<Aircraft>>,
    input: Res<InputAxis>,
    state: Res<AircraftState>,
) {
    let canards_angle = super::mechanics::canards_angle(aircraft, *state);

    let aileron_angle = (input.roll * 30.0).to_radians();
    let elevator_angle = (-input.pitch * 30.0).to_radians();

    let lerp_speed = 0.05;

    for (mut transform, ctrl_surface) in ctrl_surfaces {
        match ctrl_surface {
            ControlSurfaces::CanardPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(canards_angle.0), lerp_speed);
            }
            ControlSurfaces::CanardStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(canards_angle.1), lerp_speed);
            }
            ControlSurfaces::Rudder => {
                transform.rotation = transform.rotation.lerp(
                    Quat::from_rotation_y((-input.yaw * 30.0).to_radians()),
                    lerp_speed,
                )
            }
            ControlSurfaces::Elevator => todo!(),
            ControlSurfaces::FlapPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(elevator_angle), lerp_speed)
            }
            ControlSurfaces::FlapStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(elevator_angle), lerp_speed)
            }
            ControlSurfaces::AileronPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(-aileron_angle), lerp_speed)
            }
            ControlSurfaces::AileronStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(aileron_angle), lerp_speed)
            }
        }
    }
}
