use crate::{Aircraft, InputAxis};
use bevy::prelude::*;
use bevy_rapier3d::prelude::ExternalForce;

pub fn aircraft_mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut force: Single<&mut ExternalForce, With<Aircraft>>,
    input: Res<InputAxis>,
) {
    force.force = transform.up() * 500000. * ((input.w + 1.) / 2.);
    force.torque = Vec3 {
        x: -input.x,
        y: input.y,
        z: -input.z,
    } * 1000000.
}
