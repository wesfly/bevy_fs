use crate::{
    Settings,
    scenery::water::{Water, spawn_water},
    ui::MenuCamera,
};
use avian3d::prelude::{
    ColliderConstructor, ColliderConstructorHierarchy, Friction, Restitution, RigidBody, SweepMode,
    SweptCcd,
};
use bevy::{
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    pbr::ExtendedMaterial,
    prelude::*,
};

pub mod terrain;
pub mod water;

pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    water_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>>,
    camera: Single<Entity, With<MenuCamera>>,
) {
    commands.entity(*camera).despawn();

    if let Some(material) = water_materials {
        spawn_water(&mut commands, &asset_server, &mut meshes, material);
    }

    // runway
    commands.spawn((
        SceneRoot(asset_server.load("models/scenery/rwy/rwy.gltf#Scene0")),
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
        Restitution::new(0.0),
        SweptCcd::new_with_mode(SweepMode::Linear),
        Friction::new(1.0),
        Transform {
            translation: Vec3::new(-3000.0, 60.0, 0.0),
            rotation: Quat::from_rotation_z(-0.9_f32.to_radians()),
            ..default()
        },
    ));

    let hospital_spawn_pos = Vec3::new(0.0, 0.0, 0.0);

    commands.spawn((
        SceneRoot(asset_server.load("models/scenery/hospital/hospital.gltf#Scene0")),
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        Transform::from_translation(hospital_spawn_pos),
    ));

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: settings.shadow_distance,
        ..Default::default()
    }
    .build();

    let sun_position = settings.sun_position;
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_translation(sun_position).looking_at(Vec3::ZERO, Vec3::Y),
        cascade,
    ));
}
