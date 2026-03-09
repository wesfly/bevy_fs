use crate::aircraft::{Aircraft, AircraftState, ControlSurfaces, RotorTypes};
use crate::input::InputAxis;
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

pub fn update_rotors(
    query: Query<(&mut Transform, &RotorTypes)>,
    state: Res<AircraftState>,
    time: Res<Time>,
) {
    if state.engine_on {
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
    aircraft: Single<(&Transform, &LinearVelocity), With<Aircraft>>,
    input: Res<InputAxis>,
) {
    let velocity_dir = aircraft.1.to_vec3a().to_vec3();
    let transform = aircraft.0;
    let sin = transform
        .forward()
        .cross(velocity_dir)
        .dot(transform.right().as_vec3());
    let cos = transform.forward().dot(velocity_dir);
    let aoa = -sin.atan2(cos).to_degrees();

    let canards_angle;
    if velocity_dir.length() <= 1.0 {
        canards_angle = 0.0
    } else {
        canards_angle = aoa.clamp(-50.0, 50.0).to_radians()
    }

    let aileron_angle = (input.roll * 40.0).to_radians();

    let elevator_angle = (-input.pitch * 40.0).to_radians();

    let lerp_speed = 0.05;

    for (mut transform, ctrl_surface) in ctrl_surfaces {
        match ctrl_surface {
            ControlSurfaces::CanardPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(canards_angle), lerp_speed);
            }
            ControlSurfaces::CanardStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(canards_angle), lerp_speed);
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
                    .lerp(Quat::from_rotation_x(aileron_angle), lerp_speed)
            }
            ControlSurfaces::AileronStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(-aileron_angle), lerp_speed)
            }
        }
    }
}
