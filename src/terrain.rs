// Once again, Chis Biscardi saved me here. Without him, I'd probably still be struggling.

use crate::Settings;
use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize)]
struct Chunk {
    coordinates: Vec2,
    height_data: Vec<f32>,
}

#[derive(Serialize, Deserialize)]
struct Coordinates {
    lat: f32,
    long: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Terrain {
    collisions: bool,
    number_of_chunks: u32,
    subdivisions_per_chunk: u32,
    coordinates: Coordinates,
}

pub fn spawn_terrain(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_data: ResMut<TerrainData>,
    settings: &Res<Settings>,
) {
    let subdivisions = settings.terrain.number_of_chunks.isqrt(); // Subdivisions per axis for chunking
    let subdivisions_per_chunk: u32 = settings.terrain.subdivisions_per_chunk;
    const MESH_SIZE: f32 = 20_000.0;
    const TERRAIN_HEIGHT_FACTOR: f32 = 0.2;
    const TERRAIN_SCALE: f32 = 0.0001;

    let fetched_data: Option<Vec<Chunk>>;
    let file_exists: bool;
    if let Ok(json_data) = std::fs::read_to_string("terrain.json") {
        fetched_data = serde_json::from_str(&json_data).unwrap();
        file_exists = true;
    } else {
        file_exists = false;
        fetched_data = None;
    }

    let mut chunk_buffer: Vec<Chunk> = vec![];
    for chunk_index in 0..subdivisions.pow(2) {
        let mut terrain = Mesh::from(
            Plane3d::default()
                .mesh()
                .size(
                    MESH_SIZE / subdivisions as f32,
                    MESH_SIZE / subdivisions as f32,
                )
                .subdivisions(subdivisions_per_chunk),
        );

        if let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
            terrain.try_attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
        {
            // Positioning chunks
            for pos in positions.iter_mut() {
                pos[0] += ((chunk_index % subdivisions) as f32 * MESH_SIZE / subdivisions as f32)
                    - MESH_SIZE / 2.0;
                pos[2] += ((chunk_index / subdivisions) as f32 * MESH_SIZE / subdivisions as f32)
                    - MESH_SIZE / 2.0;
            }

            if file_exists {
                for (i, pos) in positions.iter_mut().enumerate() {
                    pos[1] = TERRAIN_HEIGHT_FACTOR
                        * *fetched_data
                            // TODO fix this mess
                            .iter()
                            .next()
                            .unwrap()
                            .get(chunk_index as usize)
                            .unwrap()
                            .height_data
                            .get(i)
                            .unwrap_or(&0.0);
                }
            } else {
                let mut buffer: Vec<[f32; 2]> = vec![];
                let midpoint_coords = Vec2::new(
                    settings.terrain.coordinates.lat,
                    settings.terrain.coordinates.long,
                );
                info!(
                    "Fetching chunk {} of {}...",
                    chunk_index,
                    subdivisions.pow(2)
                );

                let mut coords;
                for pos in positions.iter_mut() {
                    coords = Vec2::new(pos[0], pos[2]) * TERRAIN_SCALE + midpoint_coords;
                    buffer.push(coords.to_array());
                }

                if let Ok(height_list) = get_elev(buffer, &mut terrain_data) {
                    assert_eq!(positions.len(), height_list.len());

                    for (pos, height) in positions.iter_mut().zip(&height_list) {
                        pos[1] = height * TERRAIN_HEIGHT_FACTOR;
                    }
                    chunk_buffer.push(Chunk {
                        coordinates: midpoint_coords,
                        height_data: height_list,
                    });
                }
            }
        }

        terrain.compute_normals();
        commands.spawn((
            Mesh3d(meshes.add(terrain.clone())),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: bevy::color::palettes::css::GREEN.into(),
                perceptual_roughness: 1.0,
                ..Default::default()
            })),
            Transform::from_translation(Vec3 {
                x: 0.0,
                y: -0.1,
                z: 0.0,
            }),
        ));

        if settings.terrain.collisions {
            spawn_terrain_collider(commands, terrain);
        }
    }

    if !file_exists {
        let mut file = std::fs::File::create("terrain.json").unwrap();
        let json_data = serde_json::to_string(&chunk_buffer).unwrap();
        file.write_all(json_data.as_bytes()).unwrap();
    }
}

fn spawn_terrain_collider(commands: &mut Commands, terrain: Mesh) {
    commands.spawn((
        RigidBody::Static,
        Collider::trimesh_from_mesh(&terrain).unwrap(),
    ));
}

#[derive(Resource, Serialize, Deserialize)]
pub struct TerrainData(pub Vec<f32>);

#[derive(Serialize, Deserialize)]
struct Response {
    elevations: Vec<Option<f32>>,
}

// Thanks to Frank Villaro-Dixon, the guy that provides this API
fn get_elev(coords: Vec<[f32; 2]>, data: &mut ResMut<TerrainData>) -> Result<Vec<f32>> {
    const MAX_PUSH_LEN: usize = 512;

    let mut result = vec![];
    let mut buffer = vec![];
    for (i, coord) in coords.iter().enumerate() {
        if buffer.len() < MAX_PUSH_LEN {
            buffer.push(coord);
            if i == coords.len() - 1 {
                let get_string =
                    format!("https://www.elevation-api.eu/v1/elevation?pts={buffer:?}");

                let resp = match reqwest::blocking::get(get_string.trim()) {
                    Ok(resp) => resp.text()?,
                    Err(err) => panic!("Error while fetching terrain: {err}"),
                };

                let response: Response = serde_json::from_str(&resp)?;

                let elevations = response.elevations;
                for o_elevation in &elevations {
                    let elevation = o_elevation.unwrap_or(0.0);
                    data.0.push(elevation);
                    result.push(elevation);
                }
            }
        } else {
            let get_string = format!("https://www.elevation-api.eu/v1/elevation?pts={buffer:?}");

            let resp = match reqwest::blocking::get(get_string.trim()) {
                Ok(resp) => resp.text()?,
                Err(err) => panic!("Error: {err}"),
            };

            let response: Response = serde_json::from_str(&resp)?;

            let elevations = response.elevations;
            for o_elevation in &elevations {
                let elevation = o_elevation.unwrap_or(0.0);
                data.0.push(elevation);
                result.push(elevation);
            }

            buffer.clear();
            buffer.push(coord);
        }
    }

    Ok(result)
}
