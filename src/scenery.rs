use crate::{
    CELL_SIZE, Settings,
    scenery::{
        terrain::{EARTH_RADIUS, TerrainCacheResource, coord_to_pos, init_terrain_cache},
        water::{Water, spawn_water},
    },
    ui::MenuCamera,
};
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, Friction, Restitution, RigidBody, SweepMode,
    SweptCcd,
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
        // runway
        let rwy_translation = terrain::coord_to_pos(settings.terrain.coord) * EARTH_RADIUS;
        let (rwy_cell, rwy_offset) = root.grid().translation_to_grid(rwy_translation);

        root.spawn_spatial((
            WorldAssetRoot(asset_server.load("scenery/rwy/rwy.gltf#Scene0")),
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            RigidBody::Static,
            Restitution::new(0.0),
            SweptCcd::new_with_mode(SweepMode::Linear),
            Friction::new(1.0),
            Transform::from_translation(rwy_offset),
            rwy_cell,
        ));

        let (cell, offset) = root
            .grid()
            .translation_to_grid(coord_to_pos(settings.terrain.coord));
        // let hospital_spawn_pos = Vec3::new(0.0, 0.0, 0.0);
        root.spawn_spatial((
            cell,
            WorldAssetRoot(asset_server.load("scenery/hospital/hospital.gltf#Scene0")),
            RigidBody::Static,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            // Transform::from_translation(offset),
        ));

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
        info!("{terrain_settings:?}");
        for normal in normals {
            terrain::spawn_chunk(
                &mut root,
                normal,
                // normals[1],
                &client,
                Arc::clone(&semaphore),
                cache.cache.clone(),
                terrain_settings,
            );
        }
    });

    commands.entity(*camera).despawn();

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: settings.shadow_distance,
        ..Default::default()
    }
    .build();

    let sun_position = settings.sun_position;
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            contact_shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_translation(sun_position).looking_at(Vec3::ZERO, Vec3::Y),
        cascade,
    ));
}
