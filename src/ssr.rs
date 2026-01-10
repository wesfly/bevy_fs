use crate::Settings;
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
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

// A custom [`ExtendedMaterial`] that creates animated water ripples.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Water {
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
pub struct WaterSettings {
    // How much to displace each octave each frame, in the u and v directions.
    // Two octaves are packed into each `vec4`.
    octave_vectors: [Vec4; 2],
    // How wide the waves are in each octave.
    octave_scales: Vec4,
    // How high the waves are in each octave.
    octave_strengths: Vec4,
}

const SHADER_ASSET_PATH: &str = "shaders/water_material.wgsl";

pub fn ssr_config(settings: &Settings) -> Option<(ScreenSpaceReflections, Msaa, Fxaa)> {
    if settings.ssr {
        Some((
            ScreenSpaceReflections::default(),
            Msaa::Off,
            Fxaa::default(),
        ))
    } else {
        None
    }
}

pub fn spawn_water(
    commands: &mut Commands,
    // ssr
    asset_server: &Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
) {
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
}

pub fn insert_ssr_resources(app: &mut App) {
    app.insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());
}
