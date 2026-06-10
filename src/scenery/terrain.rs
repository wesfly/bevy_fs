use crate::{Settings, camera::Camera};
use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    pbr::ExtendedMaterial,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};
pub use material::TerrainMaterial;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    f32::consts::PI,
    fs::{self, File},
    io::Write,
    path::Path,
};
use tokio::runtime::Runtime;

mod material;

#[derive(Deserialize)]
pub struct TerrainSettings {
    pub coordinates: Coordinates,
    max_render_distance: f32,
    pub level_of_detail: u8,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Coordinates {
    pub lat: f32,
    pub long: f32,
}

#[derive(Clone, Debug)]
pub struct TerrariumCoords {
    z: u8,
    x: u32,
    y: u32,
}

#[derive(Component, Clone, Debug)]
pub struct Chunk(u32, u32);

impl Coordinates {
    fn to_terrarium_coords(self, zoom: u8) -> TerrariumCoords {
        let z = zoom as f32;
        let n = 2.0_f32.powf(z);

        let x = n * ((self.long + 180.0) / 360.0);
        let lat_rad = (self.lat).to_radians();
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

const BASE_SIZE: f32 = 2048.0;

#[derive(Component)]
pub struct SpawnTerrain(Task<Option<(Mesh, Collider)>>, TerrariumCoords);

static TOKIO_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create Tokio runtime"));

fn chunk_size(lod: &u8) -> f32 {
    let chunk_size = if *lod <= 14 {
        let scale_factor = 2.0_f32.powi((14 - lod) as i32);
        BASE_SIZE * scale_factor
    } else {
        let scale_factor = 2.0_f32.powi((lod - 14) as i32);
        BASE_SIZE / scale_factor
    };
    chunk_size
}

pub fn dynamic_chunks(
    mut commands: Commands,
    settings: Res<Settings>,
    camera: Single<&Transform, With<Camera>>, // This system runs as soon as there's a Camera
    mut loaded_chunks: ResMut<LoadedChunks>,
    chunks: Query<(Entity, &Chunk, &Transform)>,
    mut messages: MessageWriter<ChunkMessage>,
) {
    let chunk_size = chunk_size(&settings.terrain.level_of_detail);

    let coords = settings
        .terrain
        .coordinates
        .clone()
        .to_terrarium_coords(settings.terrain.level_of_detail);

    // The chunk where the camera is
    let camera_chunk_x = (camera.translation.x / chunk_size) as i32;
    let camera_chunk_y = (camera.translation.z / chunk_size) as i32;

    let radius = (settings.terrain.max_render_distance / chunk_size) as i32; // The radius where the system considers to spawn chunks, but it's filtered in poll_terrain

    for x in -radius..radius {
        for y in -radius..radius {
            let coord_x = coords.x.wrapping_add((camera_chunk_x + x) as u32);
            let coord_y = coords.y.wrapping_add((camera_chunk_y + y) as u32);

            let chunk_key = (coord_x, coord_y);

            if !loaded_chunks.0.contains(&chunk_key) {
                let thread_pool = AsyncComputeTaskPool::get();

                let chunk_coord = TerrariumCoords {
                    z: settings.terrain.level_of_detail,
                    x: coord_x,
                    y: coord_y,
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

    // Despawn old chunks
    for (entity, chunk, transform) in chunks {
        // To end spawn/despawn wars with chunk_size added
        if transform.translation.distance(camera.translation)
            >= settings.terrain.max_render_distance + chunk_size
        {
            messages.write(ChunkMessage(ChunkMessages::Despawn(entity, chunk.clone())));
        }
    }
}

pub enum ChunkMessages {
    Spawn(Vec3, Mesh, Collider, TerrariumCoords),
    Despawn(Entity, Chunk),
}

#[derive(Message)]
pub struct ChunkMessage(ChunkMessages);

#[derive(Resource, Default)]
pub struct LoadedChunks(pub HashSet<(u32, u32)>);

pub fn poll_terrain(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SpawnTerrain)>,
    settings: Res<Settings>,
    mut messages: MessageWriter<ChunkMessage>,
) {
    let chunk_size = chunk_size(&settings.terrain.level_of_detail);

    let base_coords = settings
        .terrain
        .coordinates
        .clone()
        .to_terrarium_coords(settings.terrain.level_of_detail);

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

                messages.write(ChunkMessage(ChunkMessages::Spawn(
                    translation,
                    mesh,
                    collider,
                    coord,
                )));
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
        mut terrain_materials: Option<
            ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
        >,
        mut loaded_chunks: ResMut<LoadedChunks>,
        asset_server: Res<AssetServer>,
    ) {
        for message in messages.read() {
            match &message.0 {
                ChunkMessages::Spawn(translation, mesh, collider, coord) => {
                    if let Some(ref mut material) = terrain_materials {
                        Chunk::spawn(
                            &mut commands,
                            &mut meshes,
                            material,
                            mesh.clone(),
                            collider.clone(),
                            translation,
                            coord,
                            &asset_server,
                        );
                    } else {
                        warn!("whoops")
                    }
                }
                ChunkMessages::Despawn(entity, coord) => {
                    Self::despawn((entity, coord), &mut commands, &mut loaded_chunks);
                    info!("Despawning chunks: {coord:?}")
                }
            };
        }
    }

    fn spawn(
        commands: &mut Commands<'_, '_>,
        meshes: &mut ResMut<'_, Assets<Mesh>>,
        terrain_materials: &mut ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
        mesh: Mesh,
        collider: Collider,
        translation: &Vec3,
        coord: &TerrariumCoords,
        asset_server: &Res<AssetServer>,
    ) {
        commands.spawn((
            collider,
            RigidBody::Static,
            Transform::from_translation(*translation),
            Mesh3d(meshes.add(mesh)),
            Chunk(coord.x, coord.y),
            MeshMaterial3d(terrain_materials.add(ExtendedMaterial {
                base: StandardMaterial {
                    base_color: Color::BLACK,
                    perceptual_roughness: 1.0,
                    ..Default::default()
                },
                extension: TerrainMaterial {
                    normals: asset_server.load_with_settings::<Image, ImageLoaderSettings>(
                        "textures/water_normals.png",
                        |settings| {
                            settings.is_srgb = false;
                            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                                address_mode_u: ImageAddressMode::Repeat,
                                address_mode_v: ImageAddressMode::Repeat,
                                mag_filter: ImageFilterMode::Linear,
                                min_filter: ImageFilterMode::Linear,
                                ..default()
                            });
                        },
                    ),
                },
            })),
        ));
    }

    fn despawn(
        entity: (&Entity, &Chunk),
        commands: &mut Commands,
        loaded_chunks: &mut ResMut<LoadedChunks>,
    ) {
        loaded_chunks.0.remove(&(entity.1.0, entity.1.1));
        commands.entity(*entity.0).despawn();
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
        Err(_) => {
            warn!("Failed to load WebP image, skipping");
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
