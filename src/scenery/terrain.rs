use crate::{Settings, camera::Camera};
use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
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
            z: zoom,
            x: tile_x,
            y: tile_y,
        }
    }
}

const CHUNKS_PER_SIDE: u32 = 5;
const BASE_SIZE: f32 = 2048.0;

pub fn spawn_terrain(mut commands: Commands) {
    commands.init_resource::<LoadedChunks>();
}

#[derive(Component)]
pub struct SpawnTerrain(Task<Option<(Mesh, Collider)>>, TerrariumCoords);

static TOKIO_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

pub fn load_dynamic_chunks(
    mut commands: Commands,
    settings: Res<Settings>,
    camera: Single<&Transform, With<Camera>>,
    mut loaded_chunks: ResMut<LoadedChunks>,
) {
    let chunk_size = if settings.terrain.level_of_detail <= 14 {
        let scale_factor = 2.0_f32.powi((14 - settings.terrain.level_of_detail) as i32);
        BASE_SIZE * scale_factor
    } else {
        let scale_factor = 2.0_f32.powi((settings.terrain.level_of_detail - 14) as i32);
        BASE_SIZE / scale_factor
    };

    let coords = Coordinates::to_terrarium_coords(
        &settings.terrain.coordinates,
        settings.terrain.level_of_detail,
    );
    // The chunk where the camera is
    let camera_chunk_x = (camera.translation.x / chunk_size).round() as i32;
    let camera_chunk_y = (camera.translation.z / chunk_size).round() as i32;

    let radius = (CHUNKS_PER_SIDE / 2) as i32;

    for x in -radius..radius {
        for y in -radius..radius {
            let terrarium_x = coords.x.wrapping_add((camera_chunk_x + x) as u32);
            let terrarium_y = coords.y.wrapping_add((camera_chunk_y + y) as u32);

            let chunk_key = (terrarium_x, terrarium_y);

            if !loaded_chunks.0.contains(&chunk_key) {
                let thread_pool = AsyncComputeTaskPool::get();

                let chunk_coord = TerrariumCoords {
                    z: settings.terrain.level_of_detail,
                    x: terrarium_x,
                    y: terrarium_y,
                };

                let tokio_handle =
                    TOKIO_RUNTIME.spawn(get_terrain(chunk_coord.clone(), chunk_size));

                let task = thread_pool.spawn(async move {
                    match tokio_handle.await {
                        Ok(terrain) => terrain,
                        Err(e) => panic!("make this an issue pls thx:\n {:#?}", e),
                    }
                });

                commands.spawn(SpawnTerrain(task, chunk_coord.clone()));
                loaded_chunks.0.insert(chunk_key);
            }
        }
    }
}

pub enum ChunkMessages {
    Spawn(Vec3, Mesh, Collider),
    Despawn(Entity),
}

#[derive(Message)]
pub struct ChunkMessage(ChunkMessages);

#[derive(Resource, Default)]
pub struct LoadedChunks(pub HashSet<(u32, u32)>);

pub fn poll_terrain(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SpawnTerrain)>,
    settings: Res<Settings>,
    camera: Single<&Transform, With<Camera>>,
    mut messages: MessageWriter<ChunkMessage>,
) {
    let chunk_size = if settings.terrain.level_of_detail <= 14 {
        let scale_factor = 2.0_f32.powi((14 - settings.terrain.level_of_detail) as i32);
        BASE_SIZE * scale_factor
    } else {
        let scale_factor = 2.0_f32.powi((settings.terrain.level_of_detail - 14) as i32);
        BASE_SIZE / scale_factor
    };

    let base_coords = Coordinates::to_terrarium_coords(
        &settings.terrain.coordinates,
        settings.terrain.level_of_detail,
    );
    let base_world_x = base_coords.x as f32 * chunk_size;
    let base_world_y = base_coords.y as f32 * chunk_size;

    for (entity, mut task) in &mut tasks {
        if let Some(mesh_collider) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some((mesh, collider)) = mesh_collider {
                let coord = task.1.clone();

                // Calculate world position of this chunk
                let chunk_world_x = coord.x as f32 * chunk_size;
                let chunk_world_y = coord.y as f32 * chunk_size;

                let translation = Vec3::new(
                    (chunk_world_x - base_world_x) + chunk_size * 0.5,
                    0.0,
                    (chunk_world_y - base_world_y) + chunk_size * 0.5,
                );

                // Check if terrain is in render distance
                if translation.distance(camera.translation) <= settings.terrain.max_render_distance
                {
                    messages.write(ChunkMessage(ChunkMessages::Spawn(
                        translation,
                        mesh,
                        collider,
                    )));
                }
            }

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
                ChunkMessages::Spawn(translation, mesh, collider) => {
                    Chunk::spawn(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        mesh.clone(),
                        collider.clone(),
                        translation,
                    )
                    .unwrap_or(());
                }
                ChunkMessages::Despawn(entity) => {
                    Self::despawn(entity, &mut commands);
                    info!("Despawning chunks: {entity}")
                }
            };
        }
    }

    fn spawn(
        commands: &mut Commands<'_, '_>,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        materials: &mut ResMut<'_, Assets<StandardMaterial>>,
        mesh: Mesh,
        collider: Collider,
        translation: &Vec3,
    ) -> Result<()> {
        commands.spawn((
            collider,
            RigidBody::Static,
            Transform::from_translation(*translation),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: bevy::color::palettes::css::GREEN.into(),
                perceptual_roughness: 1.0,
                ..Default::default()
            })),
        ));

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

async fn get_terrain(coord: TerrariumCoords, chunk_size: f32) -> Option<(Mesh, Collider)> {
    data_from_file(&coord).await;

    let path = format!("terrain_cache/{}_{}_{}.webp", coord.z, coord.x, coord.y);
    let img = match image::load_from_memory_with_format(
        &fs::read(&path).expect("file to be fetched"),
        image::ImageFormat::WebP,
    ) {
        Ok(img) => img,
        Err(e) => {
            warn!("Failed to load WebP image: {}, {path}", e);
            return None;
        }
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
    if !heights.windows(2).all(|w| w[0] == w[1]) {
        let mut terrain = Mesh::from(
            Plane3d::default()
                .mesh()
                .size(chunk_size, chunk_size)
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

        let mesh: Mesh = terrain;

        let collider = Collider::trimesh_from_mesh(&mesh).unwrap();
        return Some((mesh, collider));
    }
    None
}

async fn data_from_file(coord: &TerrariumCoords) {
    let file_name = format!("terrain_cache/{}_{}_{}.webp", coord.z, coord.x, coord.y);
    if !Path::new(&file_name).exists() {
        let url = format!(
            "https://tiles.mapterhorn.com/{}/{}/{}.webp",
            coord.z, coord.x, coord.y
        );

        let response = reqwest::get(url).await.unwrap();
        let bytes = response.bytes().await.unwrap();

        let mut file = File::create(file_name).unwrap();
        file.write(&bytes).unwrap();
    }
}
