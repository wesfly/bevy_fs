use crate::{Aircraft, AircraftVelocity, InputAxis};
use bevy::{math::VectorSpace, prelude::*};

pub fn aircraft_mechanics(
    mut transform: Single<&mut Transform, With<Aircraft>>,
    time: Res<Time>,
    input: Res<InputAxis>,
    mut vel: ResMut<AircraftVelocity>,
) {
    let delta = time.delta_secs();
    let mass = 20000.;

    let lift = 200000.;
    let gravity = Vec3 {
        x: 0.,
        y: -9.81,
        z: 0.,
    };
    let mut forces = Vec3::ZERO;
    forces += lift * transform.up();

    forces += gravity * mass;

    let accel = forces / mass;
    vel.0 += accel * delta;

    let rotation_delta = Quat::from_euler(EulerRot::XYZ, -input.x * delta, 0., -input.z * delta);
    transform.rotate_local(rotation_delta);

    transform.translation += vel.0;
}
