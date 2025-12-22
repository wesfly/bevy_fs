use crate::{Aircraft, InputAxis, RotationOfSubject};
use bevy::prelude::*;

pub fn aircraft_mechanics(
    mut transform: Single<&mut Transform, With<Aircraft>>,
    time: Res<Time>,
    input: Res<InputAxis>,
    mut rotation: ResMut<RotationOfSubject>,
) {
    let delta = time.delta_secs();

    let rotation_x = Quat::from_rotation_x(input.x * delta);
    let rotation_z = Quat::from_rotation_z(input.z * delta);
    transform.rotate_local(rotation_x);
    transform.rotate_local(rotation_z);

    let forward = transform.back();
    transform.translation += forward * delta * input.w * 10.;

    rotation.0 = transform.rotation;
}
