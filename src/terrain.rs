// Once again, Chis Biscardi saved me here. Without him, I'd probably still be struggling.

use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;

pub fn spawn_terrain(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_data: ResMut<TerrainData>,
) {
    let mesh_size = 1000.0;
    let mut terrain = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(mesh_size, mesh_size)
            .subdivisions(128),
    );

    let fetched_data: Option<Vec<f32>>;
    let file_exists: bool;
    if let Ok(json_data) = std::fs::read_to_string("terrain.json") {
        fetched_data = serde_json::from_str(&json_data).unwrap();
        file_exists = true;
    } else {
        file_exists = false;
        fetched_data = None;
    }

    if let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
        terrain.try_attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
    {
        let mut i = 0;
        if file_exists {
            for pos in positions.iter_mut() {
                pos[1] = 0.05 * fetched_data.as_ref().unwrap()[i];
                i += 1;
            }
        } else {
            let midpoint_coords = Vec2::new(-42.163, 146.646);
            info!("Get yourself a coffe, this will take a while.");
            for pos in positions.iter_mut() {
                let coords = Vec2::new(pos[0], pos[2]) * 0.005 + midpoint_coords;
                let height = get_elev(coords, &mut terrain_data);
                pos[1] = height * 0.05;
            }

            let mut file = std::fs::File::create("terrain.json").unwrap();
            let json_data = serde_json::to_string(&terrain_data.0).unwrap();
            file.write_all(json_data.as_bytes()).unwrap();
        }
    }

    terrain.compute_normals();

    commands.spawn((
        Mesh3d(meshes.add(terrain)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: bevy::color::palettes::css::GREEN_YELLOW.into(),
            ..Default::default()
        })),
        Transform::from_translation(Vec3 {
            x: 0.0,
            y: -0.1,
            z: 0.0,
        }),
    ));
    commands.spawn((RigidBody::Static, Collider::cuboid(1000.0, 20.0, 1000.0)));
}

#[derive(Resource, Serialize, Deserialize)]
pub struct TerrainData(pub Vec<f32>);

fn get_elev(pos: Vec2, data: &mut ResMut<TerrainData>) -> f32 {
    let lat = pos.x;
    let long = pos.y;

    // Thanks to Frank Villaro-Dixon, the guy that provides this API
    // TODO Multi-coordinates
    let get_string = format!("https://www.elevation-api.eu/v1/elevation/{lat}/{long}");
    let resp = match reqwest::blocking::get(get_string) {
        Ok(resp) => resp.text().unwrap(),
        Err(err) => panic!("Error: {}", err),
    };

    let elev = resp
        .trim()
        .parse::<f32>()
        .expect("Parsing of terrain data from API failed.");
    data.0.push(elev);
    elev
}
