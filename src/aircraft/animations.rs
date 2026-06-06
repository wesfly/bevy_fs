use crate::aircraft::{Aircraft, AircraftState, BothSides, ControlSurfaces, RotorTypes};
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
    state: Res<AircraftState>,
) {
    let cs = state.control_surfaces;

    let aileron_angle = BothSides {
        port: (cs.aileron.port * 30.0).to_radians(),
        starboard: (cs.aileron.starboard * 30.0).to_radians(),
    };
    let elevator_angle = BothSides {
        port: (cs.elevator.port * 30.0).to_radians(),
        starboard: (cs.elevator.port * 30.0).to_radians(),
    };

    let lerp_speed = 0.05;

    for (mut transform, ctrl_surface) in ctrl_surfaces {
        match ctrl_surface {
            ControlSurfaces::CanardPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_y(cs.canards.port), lerp_speed);
            }
            ControlSurfaces::CanardStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_y(cs.canards.starboard), lerp_speed);
            }
            ControlSurfaces::Rudder => {
                transform.rotation = transform.rotation.lerp(
                    Quat::from_rotation_y((-cs.rudder * 30.0).to_radians()),
                    lerp_speed,
                )
            }
            ControlSurfaces::ElevatorPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(elevator_angle.port), lerp_speed)
            }
            ControlSurfaces::ElevatorStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(elevator_angle.starboard), lerp_speed)
            }

            ControlSurfaces::FlapPort => todo!(),
            ControlSurfaces::FlapStarboard => todo!(),

            ControlSurfaces::AileronPort => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(aileron_angle.port), lerp_speed)
            }
            ControlSurfaces::AileronStarboard => {
                transform.rotation = transform
                    .rotation
                    .lerp(Quat::from_rotation_x(aileron_angle.starboard), lerp_speed)
            }
        }
    }
}
