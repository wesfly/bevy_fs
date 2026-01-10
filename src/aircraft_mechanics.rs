use crate::{Aircraft, InputAxis};
use avian3d::prelude::*;
use bevy::prelude::*;

pub fn aircraft_mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut query: Query<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
) {
    let force = transform.up() * 35000. * (input.throttle);
    let torque = Vec3::new(input.pitch, input.yaw * 2.5, input.roll);

    for mut forces in &mut query {
        forces.apply_force(force);
        forces.apply_local_torque(torque * 1200.);
    }
}
