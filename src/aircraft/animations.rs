use crate::{
    AircraftState,
    aircraft::Aircraft,
    data_from_gltf::{ControlSurfaces, RotorTypes},
};
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
) {
    let velocity_dir = aircraft.1.to_vec3a().to_vec3();
    let transform = aircraft.0;
    let sin = transform
        .forward()
        .cross(velocity_dir)
        .dot(transform.right().as_vec3());
    let cos = transform.forward().dot(velocity_dir);
    let aoa = -sin.atan2(cos).to_degrees();

    for (mut transform, ctrl_surface) in ctrl_surfaces {
        match ctrl_surface {
            ControlSurfaces::CanardPort => {
                transform.rotation = Quat::from_rotation_x(aoa * 1.0 / 90.0)
            }
            ControlSurfaces::CanardStarboard => {
                transform.rotation = Quat::from_rotation_x(aoa * 1.0 / 90.0)
            }
            ControlSurfaces::Rudder => todo!(),
            ControlSurfaces::Elevator => todo!(),
        }
    }
}
