use crate::{Aircraft, InputAxis};
use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalForce;

pub fn aircraft_mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut force: Single<&mut ExternalForce, With<Aircraft>>,
    input: Res<InputAxis>,
) {
    force.force = transform.up() * 50_000. * ((input.throttle + 1.) / 2.);

    let local_vec = Vec3::new(input.pitch, input.yaw, input.roll);
    let world_vec = transform.rotation() * local_vec;
    force.torque = world_vec * 500_000.;
}
