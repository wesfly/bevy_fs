use crate::{Aircraft, InputAxis, RotationOfSubject};
use bevy::prelude::*;

pub fn aircraft_mechanics(
    mut query: Query<&mut Transform, With<Aircraft>>,
    time: Res<Time>,
    input: Res<InputAxis>,
    mut rotation: ResMut<RotationOfSubject>,
) {
    let delta = time.delta_secs();
    for mut transform in &mut query {
        let rotation_x = Quat::from_rotation_x(input.x * delta);
        let rotation_z = Quat::from_rotation_z(input.z * delta);
        transform.rotate_local(rotation_x);
        transform.rotate_local(rotation_z);

        let forward = transform.back();
        transform.translation += forward * delta * 10.; //* input.z; // my right stick only works on web (idk why), thats why I neglect it completely

        rotation.0 = transform.rotation;
    }
}
