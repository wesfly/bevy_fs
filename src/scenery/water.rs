use bevy::{
    image::{
        ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler,
        ImageSamplerDescriptor,
    },
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
};

// A custom [`ExtendedMaterial`] that creates animated water ripples.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct Water {
    // The normal map image.
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

pub fn spawn_water(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    meshes: &mut ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, Water>>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10000.0)))),
        MeshMaterial3d(water_materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::Srgba(Srgba {
                    red: 0.01,
                    green: 0.1,
                    blue: 0.2,
                    alpha: 1.0,
                }),
                perceptual_roughness: 0.,
                reflectance: 0.5,

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
                settings: WaterSettings {
                    octave_vectors: [
                        vec4(0.080, 0.059, 0.073, -0.062),
                        vec4(0.153, 0.138, -0.149, -0.195),
                    ],
                    octave_scales: vec4(1.0, 2.1, 7.9, 14.9) * 250.0,
                    octave_strengths: vec4(0.16, 0.18, 0.093, 0.044) * 0.2,
                },
            },
        })),
    ));
}
