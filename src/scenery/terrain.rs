use crate::Settings;
use avian3d::prelude::{ColliderConstructor, RigidBody};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    f64::consts::PI,
    fs::{self, File},
    io::Write,
    path::Path,
};
use tokio::runtime::Runtime;

#[derive(Deserialize)]
pub struct TerrainSettings {
    coordinates: Coordinates,
}

#[derive(Serialize, Deserialize, Clone)]
struct Coordinates {
    lat: f32,
    long: f32,
}

#[derive(Clone)]
pub struct TerrariumCoords {
    z: u8,
    x: u32,
    y: u32,
}

impl Coordinates {
    fn to_terrarium_coords(coords: &Self, zoom: f32) -> TerrariumCoords {
        let z = zoom as f64;
        let n = 2.0_f64.powf(z);

        let x = n * ((coords.long as f64 + 180.0) / 360.0);
        let lat_rad = (coords.lat as f64).to_radians();
        let y = (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / PI) / 2.0 * n;

        let tile_x = x.floor() as u32;
        let tile_y = y.floor() as u32;

        TerrariumCoords {
            z: zoom as u8,
            x: tile_x,
            y: tile_y,
        }
    }
}

#[derive(Component)]
pub struct SpawnTerrain(Task<()>, (usize, usize));

static TOKIO_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

pub fn spawn_terrain(
    mut commands: Commands,
    mut coord_list: ResMut<TerrainPathList>,
    settings: Res<Settings>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let chunks_per_side: u32 = 9;
    // let chunks = chunks_per_side.pow(2);

    for x in 0..chunks_per_side {
        for y in 0..chunks_per_side {
            let coords = &settings.terrain.coordinates;

            let mut coords = Coordinates::to_terrarium_coords(&coords, 14.0);
            coords.y += y as u32;
            coords.x += x as u32;
            coord_list.0.push(coords.clone());
            let tokio_handle = TOKIO_RUNTIME.spawn(get_terrain(coords));

            let task = thread_pool.spawn(async move {
                match tokio_handle.await {
                    Ok(terrain) => terrain,
                    Err(e) => panic!("make this an issue pls thx: {:?}", e),
                }
            });

            commands.spawn(SpawnTerrain(task, (x as usize, y as usize)));
        }
    }
}

const MESH_SIZE: f32 = 2_000.0;

#[derive(Resource)]
pub struct TerrainPathList(pub Vec<TerrariumCoords>);

pub fn poll_terrain(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SpawnTerrain)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    coords: Res<TerrainPathList>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(_) = future::block_on(future::poll_once(&mut task.0)) {
            let translation = Vec3::new(
                MESH_SIZE * task.1.0 as f32,
                0.0,
                MESH_SIZE * task.1.1 as f32,
            );

            let coord = &coords.0[task.1.0 * 9 + task.1.1];

            let path = format!("terrain_cache/{}_{}_{}.webp", coord.z, coord.x, coord.y);

            let webp_bytes = fs::read(&path).expect("file to be fetched");
            let Ok(img) = image::load_from_memory_with_format(
                webp_bytes.iter().as_slice(),
                image::ImageFormat::WebP,
            ) else {
                warn!("Failed to decode tile {path}");
                commands.entity(entity).remove::<SpawnTerrain>();

                break;
            };

            let rgb_img = img.to_rgb8();
            let (width, height) = rgb_img.dimensions();

            let mut heights: std::vec::Vec<f32> = Vec::with_capacity((width * height) as usize);

            for pixel in rgb_img.pixels() {
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;

                let h = (r * 256.0 + g + b / 256.0) - 32768.0;
                heights.push(h);
            }

            commands.entity(entity).remove::<SpawnTerrain>();

            // if all elements of heights are the same
            match !heights.windows(2).all(|w| w[0] == w[1]) {
                true => {
                    let mut terrain = Mesh::from(
                        Plane3d::default()
                            .mesh()
                            .size(MESH_SIZE, MESH_SIZE)
                            .subdivisions(width - 2),
                    );

                    if let bevy::mesh::VertexAttributeValues::Float32x3(positions) =
                        terrain.try_attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap()
                    {
                        assert_eq!(positions.len(), heights.len());

                        for (i, pos) in positions.iter_mut().enumerate() {
                            pos[1] = heights[i]
                        }
                    }
                    terrain.compute_normals();
                    commands.spawn((
                        Mesh3d(meshes.add(terrain)),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: bevy::color::palettes::css::GREEN.into(),
                            perceptual_roughness: 1.0,
                            ..Default::default()
                        })),
                        ColliderConstructor::TrimeshFromMesh,
                        RigidBody::Static,
                        Transform::from_translation(translation),
                    ));
                }
                false => {}
            };
        }
    }
}

async fn get_terrain(coords: TerrariumCoords) {
    let file_name = format!("terrain_cache/{}_{}_{}.webp", coords.z, coords.x, coords.y);
    if !Path::new(&file_name).exists() {
        let url = format!(
            "https://tiles.mapterhorn.com/{}/{}/{}.webp",
            coords.z, coords.x, coords.y
        );

        let response = reqwest::get(url).await.unwrap();
        let bytes = response.bytes().await.unwrap();

        let mut file = File::create(file_name).unwrap();
        file.write(&bytes).unwrap();
    }
}
