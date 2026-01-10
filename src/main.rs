/*
I made a little flight simulator here. Check out the README for further information.
If you have fixes or want to contribute, just make a pull request (unless it's AI-generated)

I don't exactly know where these InheritedVisibility warnings are coming from (they're probably from the aircraft and its hitbox), I'm just
ignoring them.
*/

mod aircraft_mechanics;
mod camera;
mod handle_custom_properties;
mod input;
mod ui;

use crate::{
    aircraft_mechanics::aircraft_mechanics,
    camera::{CameraSettings, camera_controller},
    handle_custom_properties::on_scene_spawn,
    input::GamepadSettings,
    ui::{setup_ui, update_ui},
};

use avian3d::prelude::*;

// ssr
use bevy::{
    anti_alias::fxaa::Fxaa,
    color::palettes::css::BLACK,
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    pbr::{
        DefaultOpaqueRendererMethod, ExtendedMaterial, MaterialExtension, ScreenSpaceReflections,
    },
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, light_consts::lux},
    pbr::{Atmosphere, AtmosphereMode, AtmosphereSettings},
    post_process::{bloom::Bloom, motion_blur::MotionBlur},
    prelude::*,
    render::view::Hdr,
    scene::SceneInstanceReady,
};
use serde::{Deserialize, Serialize};
use std::fs;

#[cfg(debug_assertions)]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;

#[derive(Resource, Serialize, Deserialize)]
pub struct Settings {
    gamepad_enabled: bool,
    motion_blur_enabled: bool,
    shadow_distance: f32,
}

impl Settings {
    fn fetch() -> Self {
        let json_data = fs::read_to_string("settings.json").unwrap();
        let settings: Self = serde_json::from_str(&json_data).unwrap();
        settings
    }
}

// A custom [`ExtendedMaterial`] that creates animated water ripples.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct Water {
    // The normal map image.
    //
    // Note that, like all normal maps, this must not be loaded as sRGB.
    #[texture(100)]
    #[sampler(101)]
    normals: Handle<Image>,

    // Parameters to the water shader.
    #[uniform(102)]
    settings: WaterSettings,
}

impl MaterialExtension for Water {
    fn deferred_fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

// Parameters to the water shader.
#[derive(ShaderType, Debug, Clone)]
struct WaterSettings {
    // How much to displace each octave each frame, in the u and v directions.
    // Two octaves are packed into each `vec4`.
    octave_vectors: [Vec4; 2],
    // How wide the waves are in each octave.
    octave_scales: Vec4,
    // How high the waves are in each octave.
    octave_strengths: Vec4,
}

const SHADER_ASSET_PATH: &str = "shaders/water_material.wgsl";

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct Aircraft;

#[derive(Resource)]
struct InputAxis {
    pitch: f32,    // Pitch
    yaw: f32,      // Yaw
    roll: f32,     // Roll
    throttle: f32, // Throttle
}

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(InputAxis {
            pitch: 0.,
            yaw: 0.,
            roll: 0.,
            throttle: 1.,
        })
        .insert_resource(GamepadSettings::default())
        .insert_resource(CameraSettings::default())
        .insert_resource(input::Keymap::default())
        .insert_resource(Settings::fetch())
        .add_systems(Startup, (setup, setup_ui))
        .add_systems(
            Update,
            (
                input::input_system,
                aircraft_mechanics,
                camera_controller,
                update_ui,
            ),
        );

    // ssr
    app.insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());

    #[cfg(debug_assertions)]
    app.add_plugins(FpsOverlayPlugin::default());

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    camera_settings: Res<CameraSettings>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    settings: Res<Settings>,

    // ssr
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
) {
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("aircraft.glb")),
    );
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // landscape
    commands
        .spawn(SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("landscape.glb")),
        ))
        .observe(on_scene_spawn);

    // aircraft
    let aircraft = commands
        .spawn((
            SceneRoot(asset_server.load("aircraft.glb#Scene1")),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            Transform::from_xyz(0., 20., 0.),
            Mass(5000.),
            Visibility::Hidden,
        ))
        .id();

    let mut camera = commands.spawn((
        Camera3d::default(),
        Transform::from_translation(camera_settings.follow_default_position)
            .looking_at(camera_settings.follow_default_lookat, Vec3::Y),
        Atmosphere::EARTH,
        AtmosphereSettings {
            rendering_method: AtmosphereMode::Raymarched,
            ..Default::default()
        },
        Exposure::SUNLIGHT,
        Tonemapping::AgX,
        Bloom::NATURAL,
        Projection::from(PerspectiveProjection {
            fov: 50.0_f32.to_radians(),
            ..default()
        }),
        Hdr,
        FollowCamera,
        ChildOf(aircraft),
        // ssr
        ScreenSpaceReflections::default(),
        Msaa::Off,
        Fxaa::default(),
    ));

    if let Some(mb) = motion_blur(&settings) {
        camera.insert(mb);
    }

    // ssr water plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(100.0)))),
        MeshMaterial3d(water_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: BLACK.into(),
                perceptual_roughness: 0.,
                ..default()
            },
            extension: Water {
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
                // These water settings are just random values to create some
                // variety.
                settings: WaterSettings {
                    octave_vectors: [
                        vec4(0.080, 0.059, 0.073, -0.062),
                        vec4(0.153, 0.138, -0.149, -0.195),
                    ],
                    octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 5.0,
                    octave_strengths: vec4(0.16, 0.18, 0.093, 0.044),
                },
            },
        })),
        Transform::from_scale(Vec3::splat(100.0)),
    ));

    commands
        .spawn((
            SceneRoot(asset_server.load("aircraft.glb#Scene0")),
            Visibility::Visible,
            ChildOf(aircraft),
            animation_to_play,
        ))
        .observe(play_animation_when_ready);

    let cascade = CascadeShadowConfigBuilder {
        maximum_distance: shadow_distance(&settings),
        ..Default::default()
    }
    .build();

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(2.0, 1.0, -4.0).looking_at(Vec3::ZERO, Vec3::Y),
        cascade,
    ));
}

fn play_animation_when_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(animation_to_play) = animations_to_play.get(scene_ready.entity) {
        for child in children.iter_descendants(scene_ready.entity) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(animation_to_play.index).repeat();
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}

fn motion_blur(settings: &Res<Settings>) -> Option<MotionBlur> {
    if settings.motion_blur_enabled {
        Some(MotionBlur {
            shutter_angle: 1.0,
            samples: 6,
        })
    } else {
        None
    }
}

fn shadow_distance(settings: &Res<Settings>) -> f32 {
    settings.shadow_distance
}
