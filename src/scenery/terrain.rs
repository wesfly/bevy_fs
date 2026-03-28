use crate::{Settings, camera::Camera};
use avian3d::prelude::{Collider, RigidBody};
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
    max_render_distance: f32,
    pub level_of_detail: u8,
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

#[derive(Component)]
pub struct Chunk;

impl Coordinates {
    fn to_terrarium_coords(coords: &Self, zoom: u8) -> TerrariumCoords {
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

const CHUNKS_PER_SIDE: u32 = 9;

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

    for x in 0..CHUNKS_PER_SIDE {
        for y in 0..CHUNKS_PER_SIDE {
            let coords = &settings.terrain.coordinates;

            let mut coords =
                Coordinates::to_terrarium_coords(&coords, settings.terrain.level_of_detail);

            coords.y += y as u32;
            coords.y -= CHUNKS_PER_SIDE / 2;

            coords.x += x as u32;
            coords.x -= CHUNKS_PER_SIDE / 2;

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

pub enum ChunkMessages {
    Spawn(Vec3, TerrariumCoords, f32),
    Despawn(Entity),
}

#[derive(Message)]
pub struct ChunkMessage(ChunkMessages);

#[derive(Resource)]
pub struct TerrainPathList(pub Vec<TerrariumCoords>);

pub fn poll_terrain(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SpawnTerrain)>,
    coords: Res<TerrainPathList>,
    settings: Res<Settings>,
    camera: Single<&Transform, With<Camera>>,
    mut messages: MessageWriter<ChunkMessage>,
) {
    let base_size = 2048.0;

    let chunk_size = if settings.terrain.level_of_detail <= 14 {
        let scale_factor = 2.0_f32.powi((14 - settings.terrain.level_of_detail) as i32);
        base_size * scale_factor
    } else {
        let scale_factor = 2.0_f32.powi((settings.terrain.level_of_detail - 14) as i32);
        base_size / scale_factor
    };

    for (entity, mut task) in &mut tasks {
        if let Some(_) = future::block_on(future::poll_once(&mut task.0)) {
            let translation = Vec3::new(
                chunk_size as f32 * task.1.0 as f32 - chunk_size * 0.5 * CHUNKS_PER_SIDE as f32,
                0.0,
                chunk_size as f32 * task.1.1 as f32 - chunk_size * 0.5 * CHUNKS_PER_SIDE as f32,
            );

            // Check if terrain should even be spawned
            if translation.distance(camera.translation) > settings.terrain.max_render_distance {
                commands.entity(entity).remove::<SpawnTerrain>();
                break;
            }

            let coord = &coords.0[task.1.0 * CHUNKS_PER_SIDE as usize + task.1.1];

            messages.write(ChunkMessage(ChunkMessages::Spawn(
                translation,
                coord.clone(),
                chunk_size,
            )));

            commands.entity(entity).remove::<SpawnTerrain>();
        }
    }
}

impl Chunk {
    pub fn message_reader(
        mut messages: MessageReader<ChunkMessage>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for message in messages.read() {
            match &message.0 {
                ChunkMessages::Spawn(translation, coord, chunk_size) => {
                    Chunk::spawn(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        chunk_size,
                        translation,
                        coord,
                    )
                    .unwrap_or(warn!("Failed to decode tile, skipping..."));
                }
                ChunkMessages::Despawn(entity) => {
                    Self::despawn(entity, &mut commands);
                    info!("despawning chunks: {entity}")
                }
            };
        }
    }

    fn spawn(
        commands: &mut Commands<'_, '_>,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<StandardMaterial>>,
        chunk_size: &f32,
        translation: &Vec3,
        coord: &TerrariumCoords,
    ) -> Result<()> {
        let path = format!("terrain_cache/{}_{}_{}.webp", coord.z, coord.x, coord.y);
        let Ok(img) = image::load_from_memory_with_format(
            fs::read(&path)
                .expect("file to be fetched")
                .iter()
                .as_slice(),
            image::ImageFormat::WebP,
        ) else {
            return Err("Failed to decode tile {path}".into());
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

        // unless all elements of heights are the same
        if !heights.windows(2).all(|w| w[0] == w[1]) {
            let mut terrain = Mesh::from(
                Plane3d::default()
                    .mesh()
                    .size(*chunk_size as f32, *chunk_size as f32)
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
                Collider::trimesh_from_mesh(&terrain).expect("trimesh_from_mesh to work"),
                RigidBody::Static,
                Transform::from_translation(*translation),
                Mesh3d(meshes.add(terrain)),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: bevy::color::palettes::css::GREEN.into(),
                    perceptual_roughness: 1.0,
                    ..Default::default()
                })),
                Chunk,
            ));
        } else {
            info!("all-zero, skipping chunk");
        }

        Ok(())
    }

    fn despawn(entity: &Entity, commands: &mut Commands) {
        commands.entity(*entity).despawn();
    }
}

pub fn update_chunks(
    chunks: Query<(Entity, &Transform), With<Chunk>>,
    camera: Single<&Transform, With<Camera>>,
    settings: Res<Settings>,
    mut messages: MessageWriter<ChunkMessage>,
) {
    for (entity, transform) in &chunks {
        if transform.translation.distance(camera.translation) > settings.terrain.max_render_distance
        {
            messages.write(ChunkMessage(ChunkMessages::Despawn(entity)));
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
