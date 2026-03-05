use avian3d::prelude::{ColliderConstructor, ColliderConstructorHierarchy, RigidBody};
use bevy::{
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    pbr::ExtendedMaterial,
    prelude::*,
};

use crate::{Settings, sse, ui::MenuCamera};

pub mod terrain;

pub fn setup_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings: Res<Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    water_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, sse::Water>>>>,
    camera: Single<Entity, With<MenuCamera>>,
) {
    commands.entity(*camera).despawn();

    if let Some(material) = water_materials {
        sse::spawn_water(&mut commands, &asset_server, &mut meshes, material);
    }

    commands.spawn((
        SceneRoot(asset_server.load("hospital.glb#Scene0")),
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        Transform::from_xyz(0.0, 0.0, 0.0),
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
