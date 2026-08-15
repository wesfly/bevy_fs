use crate::{EARTH_RADIUS, absolute_position};
use avian3d::prelude::*;
use bevy::{
    color::palettes::css::GREEN,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    pbr::ExtendedMaterial,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, futures_lite::future},
};
use big_space::prelude::*;
use dashmap::DashMap;
pub use material::TerrainMaterial;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Deserialize;
use std::fs::File;
use std::{collections::HashSet, f32::consts::PI, io::Write, path::Path, sync::Arc};
use tokio::runtime::Runtime;
use tokio::sync::Semaphore;

mod material;

const SIZE: f32 = 2.0;
const SUBDIV: u32 = 8192;
const CHUNKS: u32 = SUBDIV.pow(2);
const SUBDIV_PER_TILE: u32 = 64;

#[derive(Resource, Clone)]
pub struct TerrainCacheResource {
    pub cache: TileCache,
}
impl Default for TerrainCacheResource {
    fn default() -> Self {
        return Self {
            cache: Arc::new(DashMap::new()),
        };
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct TerrainSettings {
    pub coord: Coord,
    max_render_distance: f32,
    pub level_of_detail: u8,
}

impl Default for TerrainSettings {
    fn default() -> Self {
        TerrainSettings {
            coord: Coord {
                lat: 0.0,
                long: 0.0,
            },
            level_of_detail: 12,
            max_render_distance: 5000.0,
        }
    }
}

#[derive(Component)]
struct TerrainFaces;

#[derive(Component)]
pub struct SpawnTerrain(Task<Option<(Mesh, Collider)>>, (CellCoord, Vec3));

#[derive(Copy, Clone, PartialEq, Deserialize, Debug, Default)]
pub struct Coord {
    pub lat: f32,
    pub long: f32,
}

pub fn spawn_chunk(
    commands: &mut GridCommands,
    normal: Dir3,
    client: &Client,
    semaphore: Arc<Semaphore>,
    cache: TileCache,
    terrain: TerrainSettings,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let chunk_size = SIZE / SUBDIV as f32;
    let centre_offset = (SUBDIV as f32 - 1.0) * 0.5;

    // left to right,
    // top to bottom
    for i in 0..CHUNKS {
        let ix = (i % SUBDIV) as f32;
        let iy = (i / SUBDIV) as f32;

        let a = (ix - centre_offset) * chunk_size;
        let b = (iy - centre_offset) * chunk_size;

        let mut translation_per_chunk = Vec3::ZERO;
        if normal == Dir3::NEG_X || normal == Dir3::X {
            translation_per_chunk.y = a;
            translation_per_chunk.z = b;
        }
        if normal == Dir3::NEG_Y || normal == Dir3::Y {
            translation_per_chunk.x = a;
            translation_per_chunk.z = b;
        }
        if normal == Dir3::NEG_Z || normal == Dir3::Z {
            translation_per_chunk.x = a;
            translation_per_chunk.y = b;
        }

        let chunk_translation = Vec3 {
            x: normal.x,
            y: normal.y,
            z: normal.z,
        } + translation_per_chunk;

        let translation = coord_to_pos(terrain.coord);

        let projected_chunk_center = to_sphere_pos(&chunk_translation.to_array());

        if projected_chunk_center.normalize().distance(translation)
            > terrain.max_render_distance / EARTH_RADIUS
        {
            continue;
        }

        let client_clone = client.clone();
        let semaphore_clone = Arc::clone(&semaphore);

        let tokio_handle = TOKIO_RUNTIME.spawn(build_mesh(
            normal,
            chunk_translation,
            client_clone,
            semaphore_clone,
            Arc::clone(&cache),
            terrain.clone(),
        ));

        let task = thread_pool.spawn(async move { tokio_handle.await.unwrap() });
        let (cell_coord, cell_offset) = commands.grid().translation_to_grid(projected_chunk_center);
        commands.spawn(SpawnTerrain(task, (cell_coord, cell_offset)));
    }
}

/// Returns a normalized Vec3:
/// You need to multiply it with `EARTH_RADIUS` to get the absolute position
pub fn coord_to_pos(target_coord: Coord) -> Vec3 {
    let lat_rad = target_coord.lat.to_radians();
    let long_rad = target_coord.long.to_radians();

    let y = lat_rad.sin();
    let x = lat_rad.cos() * long_rad.sin();
    let z = lat_rad.cos() * long_rad.cos();
    Vec3::new(x, y, z).normalize()
}

fn to_sphere_pos(pos: &[f32; 3]) -> Vec3 {
    let p = Vec3 {
        x: pos[0],
        y: pos[1],
        z: pos[2],
    };

    let x2 = p.x * p.x;
    let y2 = p.y * p.y;
    let z2 = p.z * p.z;

    // Even spacing of vertices on sphere
    let x = p.x * (1.0 - (y2 + z2) / 2.0 + (y2 * z2 / 3.0)).sqrt();
    let y = p.y * (1.0 - (z2 + x2) / 2.0 + (z2 * x2 / 3.0)).sqrt();
    let z = p.z * (1.0 - (x2 + y2) / 2.0 + (x2 * y2 / 3.0)).sqrt();
    let even_spaced_pos = Vec3::new(x, y, z);

    even_spaced_pos * EARTH_RADIUS
}

type TileCache = Arc<DashMap<(u8, u32, u32), Arc<image::RgbImage>>>;
pub fn init_terrain_cache() -> TileCache {
    Arc::new(DashMap::new())
}

static DUMMY_TILE: Lazy<Arc<image::RgbImage>> = Lazy::new(|| {
    let mut dummy = image::RgbImage::new(512, 512);
    for pixel in dummy.pixels_mut() {
        *pixel = image::Rgb([128, 0, 0]);
    }
    Arc::new(dummy)
});

fn coord_to_tile(coord: Coord, n: f32) -> (u32, u32) {
    let x = n * ((coord.long + 180.0) / 360.0);

    let lat_rad = coord
        .lat
        .to_radians()
        .clamp(-85.05112_f32.to_radians(), 85.05112_f32.to_radians());
    let y = (1.0 - (lat_rad.tan() + (1.0 / lat_rad.cos())).ln() / std::f32::consts::PI) / 2.0 * n;

    (x.floor() as u32, y.floor() as u32)
}

async fn ensure_tiles_loaded(
    client: &Client,
    semaphore: Arc<tokio::sync::Semaphore>,
    cache: TileCache,
    required_tiles: Vec<(u8, u32, u32)>,
) {
    let mut fetch_tasks = vec![];

    for (zoom, x, y) in required_tiles {
        if cache.contains_key(&(zoom, x, y)) {
            continue;
        }

        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let cache = Arc::clone(&cache);

        let task = tokio::spawn(async move {
            let path = format!(".user/cache/{}_{}_{}.webp", zoom, x, y);

            match get_tile(&client, semaphore, &TerrariumCoords { z: zoom, x, y }).await {
                Ok(_) => {}
                Err(e) => {
                    info!("Missing tile {}/{}/{}: {}", zoom, x, y, e);
                }
            }

            let img_result = match tokio::task::spawn_blocking(move || {
                let bytes = std::fs::read(&path).unwrap_or_else(|_| vec![]);

                if bytes.is_empty() {
                    return Ok::<Arc<image::RgbImage>, String>(Arc::clone(&DUMMY_TILE));
                }

                match image::load_from_memory_with_format(&bytes, image::ImageFormat::WebP) {
                    Ok(img) => Ok(Arc::new(img.to_rgb8())),
                    Err(_) => Ok(Arc::clone(&DUMMY_TILE)),
                }
            })
            .await
            {
                Ok(img) => img,
                Err(e) => {
                    error!("Task failed: {e}");
                    Ok(Arc::clone(&DUMMY_TILE))
                }
            };

            let final_image = img_result.unwrap_or_else(|_| Arc::clone(&DUMMY_TILE));
            cache.insert((zoom, x, y), final_image);
        });

        fetch_tasks.push(task);
    }

    futures::future::join_all(fetch_tasks).await;
}

async fn get_height_at_coord(coord_: Coord, zoom: u8, cache: &TileCache) -> f32 {
    //--------------------- coords to terrarium coords ---------------------
    let coord = coord_;
    if coord.long < -180.0 || coord.long > 180.0 {
        error!("Longitude {} is out of bounds (-180.0..180.0)", coord.long);
        return 0.0;
    }

    let z = zoom as f32;
    let n = 2.0_f32.powf(z);

    let x = n * ((coord.long + 180.0) / 360.0);
    let y = (1.0 - (coord.lat.to_radians().tan() + (1.0 / coord.lat.to_radians().cos())).ln() / PI)
        / 2.0
        * n;

    // rounding down
    let tile_x = x.floor() as u32;
    let tile_y = y.floor() as u32;

    // we do all of this instead of calling the function for these two values
    let offset_x = x - tile_x as f32;
    let offset_y = y - tile_y as f32;

    //--------------------- sample elevation  ---------------------

    let px_offset_x = (offset_x * 512.0) as u32;
    let px_offset_y = (offset_y * 512.0) as u32;

    if let Some(img) = cache.get(&(zoom, tile_x, tile_y)) {
        // Double check bounds just in case of float precision weirdness
        if px_offset_x < 512 && px_offset_y < 512 {
            let pixel = img[(px_offset_x, px_offset_y)];
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            return (r * 256.0 + g + b / 256.0) - 32768.0;
        }
    }

    0.0
}

static TOKIO_RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create tokio runtime"));

async fn build_mesh(
    normal: Dir3,
    chunk_translation: Vec3,
    client: Client,
    semaphore: Arc<Semaphore>,
    cache: TileCache,
    terrain: TerrainSettings,
) -> Option<(Mesh, Collider)> {
    let n = 2.0_f32.powf(terrain.level_of_detail as f32);
    let required_tiles = calculate_required_tiles_for_chunk(chunk_translation, normal, n, &terrain);
    ensure_tiles_loaded(
        &client,
        Arc::clone(&semaphore),
        Arc::clone(&cache),
        required_tiles,
    )
    .await;
    let mut earth_mesh = Mesh::from(
        Plane3d::default()
            .mesh()
            .size(SIZE / SUBDIV as f32, SIZE / SUBDIV as f32)
            .normal(normal)
            .subdivisions(SUBDIV_PER_TILE),
    )
    .translated_by(chunk_translation);

    let projected_chunk_center = to_sphere_pos(&chunk_translation.to_array());

    // make the planes a sphere
    if let bevy::mesh::VertexAttributeValues::Float32x3(positions) = earth_mesh
        .try_attribute_mut(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
    {
        for pos in positions.iter_mut() {
            let mut even_spaced_pos = to_sphere_pos(&pos);
            *pos = (even_spaced_pos).to_array();

            let coord = pos_to_coord(*pos);

            let factor = 1.0
                + (0.0000001 * get_height_at_coord(coord, terrain.level_of_detail, &cache).await);
            even_spaced_pos.x *= factor;
            even_spaced_pos.y *= factor;
            even_spaced_pos.z *= factor;

            *pos = (even_spaced_pos - projected_chunk_center).to_array();
        }
    } else {
        return None;
    }

    earth_mesh.compute_normals();
    let collider = Collider::trimesh_from_mesh(&earth_mesh)?;

    return Some((earth_mesh, collider));
}

fn calculate_required_tiles_for_chunk(
    chunk_translation: Vec3,
    normal: Dir3,
    n: f32,
    terrain: &TerrainSettings,
) -> Vec<(u8, u32, u32)> {
    let mut unique_tiles = HashSet::new();

    let chunk_size = SIZE / SUBDIV as f32;
    let rotation = Quat::from_rotation_arc(Vec3::Y, *normal);

    let steps = SUBDIV_PER_TILE + 1;

    for i in 0..steps {
        for j in 0..steps {
            let pct_x = (i as f32 / (steps - 1) as f32) - 0.5;
            let pct_z = (j as f32 / (steps - 1) as f32) - 0.5;

            let local_pos = Vec3::new(pct_x * chunk_size, 0.0, pct_z * chunk_size);

            let world_plane_pos = rotation * local_pos + chunk_translation;
            let even_spaced_pos = to_sphere_pos(&world_plane_pos.to_array());
            let coord = pos_to_coord(even_spaced_pos.to_array());

            let (tx, ty) = coord_to_tile(coord, n);

            unique_tiles.insert((terrain.level_of_detail, tx, ty));
        }
    }

    let max_tile_limit = (2.0_f32.powi(terrain.level_of_detail as i32) as u32).saturating_sub(1);

    let mut final_tiles = Vec::new();
    for (z, tx, ty) in unique_tiles {
        // Clamp tiles to valid map ranges just in case of edge precision issues
        let clamped_x = tx.min(max_tile_limit);
        let clamped_y = ty.min(max_tile_limit);
        final_tiles.push((z, clamped_x, clamped_y));
    }

    final_tiles
}

fn pos_to_coord(pos: [f32; 3]) -> Coord {
    let distance_h = (pos[0].powi(2) + pos[2].powi(2)).sqrt();

    let bearing = pos[0].atan2(pos[2]).to_degrees();

    let elevation = pos[1]
        .atan2(distance_h)
        .to_degrees()
        .clamp(-85.05113, 85.05113);

    let coord = Coord {
        lat: elevation,
        long: bearing,
    };
    coord
}

async fn get_tile(
    client: &Client,
    semaphore: Arc<Semaphore>,
    coord: &TerrariumCoords,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_name = format!(".user/cache/{}_{}_{}.webp", coord.z, coord.x, coord.y);

    if !Path::new(&file_name).exists() {
        let _permit = semaphore.acquire().await?;

        let url = format!(
            "https://tiles.mapterhorn.com/{}/{}/{}.webp",
            coord.z, coord.x, coord.y
        );

        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(format!(
                "Server responded to {} with status: {}",
                &url,
                response.status()
            )
            .into());
        }

        let bytes = response.bytes().await?;

        let mut file = File::create(file_name)?;
        file.write_all(&bytes)?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct TerrariumCoords {
    z: u8,
    x: u32,
    y: u32,
}

pub fn poll_terrain(
    mut tasks: Query<(Entity, &mut SpawnTerrain)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
    asset_server: Res<AssetServer>,
    big_space: Single<Entity, With<BigSpace>>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mesh_collider) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some((earth_mesh, collider)) = mesh_collider {
                let (cell_coord, cell_offset) = task.1;

                let chunk = commands
                    .spawn((
                        TerrainFaces,
                        Mesh3d(meshes.add(earth_mesh)),
                        RigidBody::Static,
                        collider,
                        MeshMaterial3d(
                            terrain_materials.add(ExtendedMaterial {
                                base: StandardMaterial {
                                    base_color: Color::Srgba(GREEN),
                                    perceptual_roughness: 1.0,
                                    ..Default::default()
                                },
                                extension: TerrainMaterial {
                                    normals: asset_server
                                        .load_builder()
                                        .with_settings(|settings: &mut ImageLoaderSettings| {
                                            settings.is_srgb = false;
                                            settings.sampler =
                                                ImageSampler::Descriptor(ImageSamplerDescriptor {
                                                    address_mode_u: ImageAddressMode::Repeat,
                                                    address_mode_v: ImageAddressMode::Repeat,
                                                    mag_filter: ImageFilterMode::Linear,
                                                    min_filter: ImageFilterMode::Linear,
                                                    ..default()
                                                });
                                        })
                                        .load("textures/water_normals.png"),
                                    chunk_normal: absolute_position(&cell_coord, cell_offset)
                                        .as_vec3(),
                                },
                            }),
                        ),
                        Transform::from_translation(
                            cell_offset
                                + Vec3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: 0.0,
                                },
                        ),
                        cell_coord,
                    ))
                    .id();

                commands.entity(*big_space).add_child(chunk);
            }
            commands.entity(entity).remove::<SpawnTerrain>();
        }
    }
}
