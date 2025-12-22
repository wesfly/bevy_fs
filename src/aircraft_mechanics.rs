use crate::{Aircraft, InputAxis, RotationOfSubject};
use bevy::prelude::*;

pub fn aircraft_mechanics(
    mut transform: Single<&mut Transform, With<Aircraft>>,
    time: Res<Time>,
    input: Res<InputAxis>,
    mut rotation: ResMut<RotationOfSubject>,
) {
    let delta = time.delta_secs();

    let rotation_delta = Quat::from_euler(EulerRot::XYZ, input.x * delta, 0., input.z * delta);
    transform.rotate_local(rotation_delta);

    let forward = transform.back();
    transform.translation += forward * delta * input.w * 10.;

    rotation.0 = transform.rotation;
}
