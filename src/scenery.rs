use crate::{
    CELL_SIZE, Settings, TOKIO_RUNTIME,
    scenery::{
        osm::buildings::SpawnBuilding,
        terrain::{TerrainCacheResource, TerrainChunkRegistry},
    },
    ui::MenuCamera,
};
use bevy::{
    light::{
        Atmosphere, CascadeShadowConfigBuilder, atmosphere::ScatteringMedium, light_consts::lux,
    },
    prelude::*,
    tasks::AsyncComputeTaskPool,
};
use big_space::prelude::*;

pub mod osm;
pub mod terrain;

pub fn setup_scene(
    mut commands: Commands,
    settings: Res<Settings>,
    camera: Single<Entity, With<MenuCamera>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    mut terrain_tile_cache: ResMut<TerrainCacheResource>,
    mut terrain_registry: ResMut<TerrainChunkRegistry>,
) {
    commands.spawn_big_space(Grid::new(CELL_SIZE as f32, 10.0), |mut root| {
        let earth_medium = ScatteringMedium::default();
        let earth_atmosphere = Atmosphere::earth(scattering_mediums.add(earth_medium));
    // Init/Reset tile cache
    *terrain_tile_cache = TerrainCacheResource::default();
    *terrain_registry = TerrainChunkRegistry::default();

    let coord = settings.terrain.coord;
    let thread_pool = AsyncComputeTaskPool::get();
        root.spawn_spatial(earth_atmosphere.clone());

        if settings.buildings_enabled {
            let tokio_handle = TOKIO_RUNTIME.spawn(osm::buildings::spawn(
                root.grid().clone(),
                coord.lat - 0.05,
                coord.long - 0.05,
                coord.lat + 0.05,
                coord.long + 0.05,
            ));

            let task = thread_pool.spawn(async move { tokio_handle.await.unwrap() });
            root.spawn(SpawnBuilding { task });
        }
    });

    commands.entity(*camera).despawn();

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: settings.shadow_distance.clamp(1.0, 100_000.0),
        ..Default::default()
    }
    .build();

    let sun_position = Vec3::new(1.0, 0.1, 0.0).normalize();
    let shadows_enabled = settings.shadow_distance != 0.0;
    if !shadows_enabled {
        info!("Shadows disabled")
    }
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: shadows_enabled,
            contact_shadows_enabled: shadows_enabled,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_translation(sun_position).looking_at(Vec3::ZERO, Vec3::Y),
        cascade,
    ));
}
