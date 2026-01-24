// Once again, Chis Biscardi saved me here. Without him, I'd probably still be struggling.

use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use noiz::prelude::*;

pub fn spawn_terrain(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain_height = 70.0;
    let mut noise1 = Noise::<common_noise::Perlin>::default();
    noise1.set_period(100.0);

    let mut noise2 = Noise::<common_noise::Perlin>::default();
    noise2.set_period(200.0);

    let mesh_size = 1000.0;
    let mut terrain = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(mesh_size, mesh_size)
            .subdivisions(64),
    );

    if let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
        terrain.try_attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
    {
        for pos in positions.iter_mut() {
            let val = noise1
                .sample_for::<f32>(Vec2::new(pos[0] + (mesh_size), pos[2] + (mesh_size)))
                + noise2.sample_for::<f32>(Vec2::new(pos[0] + (mesh_size), pos[2] + (mesh_size)));
            let island_factor = Vec2::new(pos[0], pos[2]).distance(Vec2 { x: 0.0, y: 0.0 });
            pos[1] = val * terrain_height - island_factor * 0.05;
        }
    }

    terrain.compute_normals();

    commands.spawn((
        Mesh3d(meshes.add(terrain)),
        RigidBody::Static,
        Transform::from_translation(Vec3 {
            x: 0.0,
            y: 10.0,
            z: 0.0,
        }),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::color::palettes::css::GREEN_YELLOW.into(),
            ..Default::default()
        })),
    ));
    commands.spawn((RigidBody::Static, Collider::cuboid(100.0, 20.0, 100.0)));
}
