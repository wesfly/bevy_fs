/* It always fails in line 60, positions and height_list don't have the same length, but height_list.len() can be modified by changing the MAX_PUSH_LEN in get_elev, I think I messed up somewhere there. */

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
    let mesh_size = 2000.0;
    let mut terrain = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(mesh_size, mesh_size)
            .subdivisions(256),
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
        if file_exists {
            let mut i = 0;
            for pos in positions.iter_mut() {
                pos[1] = 0.01
                    * *fetched_data
                        .iter()
                        .nth(0)
                        .unwrap()
                        .iter()
                        .nth(i)
                        .unwrap_or(&0.0);
                i += 1;
            }
        } else {
            let mut buffer: Vec<[f32; 2]> = vec![];
            let midpoint_coords = Vec2::new(-42.163, 146.646);
            info!("Get yourself a coffee, this will take a while.");

            let mut coords;
            for pos in positions.iter_mut() {
                coords = Vec2::new(pos[0], pos[2]) * 0.002 + midpoint_coords;
                buffer.push(coords.to_array());
            }

            if let Ok(height_list) = get_elev(buffer, &mut terrain_data) {
                assert_eq!(positions.len(), height_list.len());

                for (pos, height) in positions.iter_mut().zip(height_list) {
                    pos[1] = height;
                }
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
    commands.spawn((RigidBody::Static, Collider::cuboid(1000.0, 1.0, 1000.0)));
}

#[derive(Resource, Serialize, Deserialize)]
pub struct TerrainData(pub Vec<f32>);

#[derive(Serialize, Deserialize)]
struct Response {
    elevations: Vec<Option<f32>>,
}

// Thanks to Frank Villaro-Dixon, the guy that provides this API
// https://www.elevation-api.eu/v1/elevation?pts=[[46.24566,6.17081],[46.85499,6.78134]]
fn get_elev(coords: Vec<[f32; 2]>, data: &mut ResMut<TerrainData>) -> Result<Vec<f32>> {
    const MAX_PUSH_LEN: usize = 512;

    let mut result = vec![];
    let mut buffer = vec![];
    for coord in coords {
        if buffer.len() < MAX_PUSH_LEN {
            buffer.push(coord)
        } else {
            let get_string = format!("https://www.elevation-api.eu/v1/elevation?pts={buffer:?}");

            let resp = match reqwest::blocking::get(get_string.trim()) {
                Ok(resp) => resp.text().unwrap(),
                Err(err) => panic!("Error: {}", err),
            };

            let response: Response;
            response = serde_json::from_str(&resp).unwrap();

            let elev = response.elevations;

            for elevation in &elev {
                match elevation {
                    Some(t) => {
                        data.0.push(*t);
                        result.push(*t);
                    }
                    _ => {}
                }
            }
            buffer.clear();
        }
    }

    info!(?result);
    Ok(result)
}
