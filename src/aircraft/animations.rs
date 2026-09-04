use crate::aircraft::{AircraftState, BothSides, ControlSurfaces, RotorTypes};
use bevy::prelude::*;

pub fn update_rotors(
    query: Query<(&mut Transform, &RotorTypes)>,
    state: Res<AircraftState>,
    time: Res<Time>,
) {
    if state.engine.on {
        for (mut rotor, rotor_type) in query {
            match rotor_type {
                RotorTypes::Main(a) => rotor.rotate_local_axis(*a, 100.0 * time.delta_secs()),
                RotorTypes::Rear => rotor.rotate_local_z(100.0 * time.delta_secs()),
            }
        }
    }
}

#[derive(Component)]
pub struct RestRotation(Quat);

pub fn update_control_surfaces(
    mut commands: Commands,
    ctrl_surfaces: Query<(
        Entity,
        &mut Transform,
        &ControlSurfaces,
        Option<&RestRotation>,
    )>,
    state: Res<AircraftState>,
) {
    let cs = state.control_surfaces;
    let aileron_angle = BothSides {
        port: (cs.aileron.port * 30.0).to_radians(),
        starboard: (cs.aileron.starboard * 30.0).to_radians(),
    };
    let elevator_angle = BothSides {
        port: (cs.elevator.port * 30.0).to_radians(),
        starboard: (cs.elevator.starboard * 30.0).to_radians(),
    };
    let lerp_speed = 0.05;

    for (entity, mut transform, ctrl_surface, rest) in ctrl_surfaces {
        // Insert RestRotation on first run
        let rest_rotation = match rest {
            Some(r) => r.0,
            None => {
                let r = transform.rotation;
                commands.entity(entity).insert(RestRotation(r));
                r
            }
        };

        let delta = match ctrl_surface {
            ControlSurfaces::CanardPort => Quat::from_rotation_y(cs.canards.port),
            ControlSurfaces::CanardStarboard => Quat::from_rotation_y(cs.canards.starboard),
            ControlSurfaces::Rudder => Quat::from_rotation_y((-cs.rudder * 30.0).to_radians()),
            ControlSurfaces::ElevatorPort => Quat::from_rotation_x(elevator_angle.port),
            ControlSurfaces::ElevatorStarboard => Quat::from_rotation_x(elevator_angle.starboard),
            ControlSurfaces::FlapPort => todo!(),
            ControlSurfaces::FlapStarboard => todo!(),
            ControlSurfaces::AileronPort => Quat::from_rotation_x(aileron_angle.port),
            ControlSurfaces::AileronStarboard => Quat::from_rotation_x(aileron_angle.starboard),
        };

        let target = rest_rotation * delta;
        transform.rotation = transform.rotation.lerp(target, lerp_speed);
    }
}
