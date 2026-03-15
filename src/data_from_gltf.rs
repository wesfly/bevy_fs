/*
This file handles importing gltfs from Blender with custom properties.

How to make buttons with custom properties (don't forget to export with custom properties enabled)
Thanks to Christopher Biscardi for making a tutorial about it.

load
=================
Buttons:
interface_type: InterfaceType
operation: InterfaceOperation
inverse: bool
-----------------
Lights:
light: Lights
-----------------
Rotors:
rotor: RotorTypes
-----------------
Shading:
not_shadow_caster: bool
-----------------
Landing Gear:
ldg_gear_element: LandingGearElements
*/

use crate::aircraft::{
    ControlSurface, Rotor,
    buttons::{Button, InterfaceType},
    landing_gear::{
        LandingGearElement,
        LandingGearElements, //, LandingGearStatus
    },
    lights::Light,
    screens::Screen,
};
use avian3d::prelude::{ColliderDisabled, Mass, RigidBodyDisabled};
use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    color::palettes::css::{BLACK, BLUE},
    gltf::GltfMeshExtras,
    light::NotShadowCaster,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
    scene::SceneInstanceReady,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ShadingFromGltf {
    not_shadow_caster: bool,
}

#[derive(Debug, Deserialize)]
struct LandingGearElementFromGltf {
    ldg_gear_element: LandingGearElements,
}

pub fn load(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mesh_extras: Query<&GltfMeshExtras>,
    other_extras: Query<&GltfExtras>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for entity in children.iter_descendants(trigger.entity.entity()) {
        if let Ok(gltf_mesh_extras) = mesh_extras.get(entity) {
            if let Ok(light_mesh_data) = serde_json::from_str::<Light>(&gltf_mesh_extras.value) {
                let material_emissive = materials.add(StandardMaterial {
                    perceptual_roughness: 0.1,
                    specular_transmission: 1.0,
                    base_color: Color::LinearRgba(LinearRgba::rgb(0.5, 0.5, 0.5).with_alpha(0.5)),
                    ..default()
                });
                commands
                    .entity(entity)
                    .insert((MeshMaterial3d(material_emissive), light_mesh_data.light));
            };

            if let Ok(rotor_data) = serde_json::from_str::<Rotor>(&gltf_mesh_extras.value) {
                commands.entity(entity).insert(rotor_data.rotor);
            };

            if let Ok(screen_data) = serde_json::from_str::<Screen>(&gltf_mesh_extras.value) {
                let size = Extent3d {
                    width: 512,
                    height: 512,
                    ..default()
                };

                // This is the texture that will be rendered to.
                let mut image = Image::new_fill(
                    size,
                    TextureDimension::D2,
                    &[0, 0, 0, 0],
                    TextureFormat::Bgra8UnormSrgb,
                    RenderAssetUsages::default(),
                );
                // You need to set these texture usage flags in order to use the image as a render target
                image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT;

                let image_handle = images.add(image);

                let texture_camera = commands
                    .spawn((
                        Camera2d,
                        Camera {
                            order: -1,
                            ..default()
                        },
                        RenderTarget::Image(image_handle.clone().into()),
                    ))
                    .id();

                commands
                    .spawn((
                        Node {
                            width: percent(100),
                            height: percent(100),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(BLACK.into()),
                        UiTargetCamera(texture_camera),
                    ))
                    .with_children(|parent| {
                        parent
                            .spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: Val::Auto,
                                    height: Val::Auto,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(20.)),
                                    border_radius: BorderRadius::all(Val::Px(10.)),
                                    ..default()
                                },
                                BackgroundColor(BLUE.into()),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(format!("{:?}", screen_data.screen)),
                                    TextFont {
                                        font_size: 40.0,
                                        ..default()
                                    },
                                    TextColor::WHITE,
                                ));
                            });
                    });

                // This material has the texture that has been rendered.
                let material_handle = materials.add(StandardMaterial {
                    base_color_texture: Some(image_handle),
                    ..default()
                });

                dbg!(&screen_data);
                commands
                    .entity(entity)
                    .insert((screen_data.screen, MeshMaterial3d(material_handle)));
            };

            if let Ok(ldg_gear_data) =
                serde_json::from_str::<LandingGearElementFromGltf>(&gltf_mesh_extras.value)
            {
                commands.entity(entity).insert((
                    LandingGearElement {
                        ldg_gear_element: ldg_gear_data.ldg_gear_element,
                    },
                    RigidBodyDisabled,
                    ColliderDisabled,
                ));
            };

            if let Ok(ctrl_surface_data) =
                serde_json::from_str::<ControlSurface>(&gltf_mesh_extras.value)
            {
                commands.entity(entity).insert((
                    ctrl_surface_data.control_surface,
                    Mass::from(0.0),
                    ColliderDisabled,
                ));
            };

            if let Ok(shading_data) =
                serde_json::from_str::<ShadingFromGltf>(&gltf_mesh_extras.value)
                && shading_data.not_shadow_caster
            {
                commands.entity(entity).insert(NotShadowCaster);
            };

            if let Ok(button_data) = serde_json::from_str::<Button>(&gltf_mesh_extras.value) {
                match button_data.interface_type {
                    InterfaceType::Button | InterfaceType::Switch => {
                        let bundle = (
                            Pickable::default(),
                            Button {
                                interface_type: button_data.interface_type,
                                inverse: button_data.inverse,
                                operation: button_data.operation,
                            },
                        );
                        commands
                            .entity(entity)
                            .insert(bundle)
                            .observe(crate::aircraft::buttons::button_listener);
                    }
                    e => {
                        warn!("{e:?} not handled yet")
                    }
                }
            };
        };

        if let Ok(gltf_other_extras) = other_extras.get(entity)
            && let Ok(light_data) = serde_json::from_str::<Light>(&gltf_other_extras.value)
        {
            commands.entity(entity).insert(light_data.light);
        };
    }
}
