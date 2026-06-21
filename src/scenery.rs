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
    light::{
        Atmosphere, CascadeShadowConfigBuilder, atmosphere::ScatteringMedium, light_consts::lux,
    },
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
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
) {
    commands.entity(*camera).despawn();

    if let Some(material) = water_materials {
        spawn_water(&mut commands, &asset_server, &mut meshes, material);
    }

    // runway
    commands.spawn((
        WorldAssetRoot(asset_server.load("scenery/rwy/rwy.gltf#Scene0")),
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
        Restitution::new(0.0),
        SweptCcd::new_with_mode(SweepMode::Linear),
        Friction::new(1.0),
        Transform::from_xyz(-3000.0, 60.0, 0.0),
    ));

    let hospital_spawn_pos = Vec3::new(0.0, 0.0, 0.0);

    commands.spawn((
        WorldAssetRoot(asset_server.load("scenery/hospital/hospital.gltf#Scene0")),
        RigidBody::Static,
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        Transform::from_translation(hospital_spawn_pos),
    ));

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: settings.shadow_distance,
        ..Default::default()
    }
    .build();

    spawn_atmosphere(&mut commands, &mut scattering_mediums);

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

fn spawn_atmosphere(
    commands: &mut Commands,
    scattering_mediums: &mut ResMut<Assets<ScatteringMedium>>,
) {
    let earth_medium = ScatteringMedium::default();
    let earth_atmosphere = Atmosphere::earth(scattering_mediums.add(earth_medium));

    commands.spawn((
        earth_atmosphere.clone(),
        Transform::from_scale(Vec3::splat(1.0))
            .with_translation(-Vec3::Y * earth_atmosphere.inner_radius),
    ));
}
