use crate::{
    CELL_SIZE, Settings,
    scenery::{
        terrain::{TerrainCacheResource, init_terrain_cache},
        water::{Water, spawn_water},
    },
    ui::MenuCamera,
};
use bevy::{
    light::{
        Atmosphere, CascadeShadowConfigBuilder, atmosphere::ScatteringMedium, light_consts::lux,
    },
    pbr::ExtendedMaterial,
    prelude::*,
};
use big_space::prelude::*;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub mod terrain;
pub mod water;

pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    water_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>>,
    camera: Single<Entity, With<MenuCamera>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    cache: Res<TerrainCacheResource>,
) {
    init_terrain_cache();
    commands.spawn_big_space(Grid::new(CELL_SIZE as f32, 10.0), |mut root| {
        let earth_medium = ScatteringMedium::default();
        let earth_atmosphere = Atmosphere::earth(scattering_mediums.add(earth_medium));

        root.spawn_spatial(earth_atmosphere.clone());

        if let Some(material) = water_materials {
            spawn_water(&mut root, &asset_server, &mut meshes, material);
        }

        let normals = vec![
            Dir3::X,
            Dir3::Y,
            Dir3::Z,
            Dir3::NEG_X,
            Dir3::NEG_Y,
            Dir3::NEG_Z,
        ];

        let client = Client::new();
        let semaphore = Arc::new(Semaphore::new(64));
        let terrain_settings = settings.terrain;
        for normal in normals {
            terrain::spawn_chunk(
                &mut root,
                normal,
                &client,
                Arc::clone(&semaphore),
                cache.cache.clone(),
                terrain_settings,
            );
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
