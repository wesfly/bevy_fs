use crate::{M_S_TO_KTS, METRES_TO_FEET, aircraft::Aircraft, input::InputAxis};
use avian3d::prelude::LinearVelocity;
use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    color::palettes::css::{BLACK, BLUE, GREEN},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Component)]
pub enum Screens {
    Left,
    Right,
    Centre,
    Hud,
}

#[derive(Deserialize, Debug)]
pub struct Screen {
    pub screen: Screens,
}

#[allow(dead_code)]
#[derive(Component)]
pub enum ScreenUiElement {
    Throttle,
    Altitude,
    AirspeedKts,
    SpeedMach,
    Alpha,
    Horizon,
}

pub fn get_material_handle(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    screen_data: &Screen,
    asset_server: &Res<AssetServer>,
) -> Handle<StandardMaterial> {
    let size = Extent3d {
        width: 1024,
        height: 1024,
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
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

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

    match screen_data.screen {
        Screens::Hud => {
            const HUD_COLOUR: Color = Color::Srgba(GREEN);

            let text_bundle = (
                TextFont {
                    font_size: 32.0,
                    font: asset_server.load("fonts/SourceCodePro-Bold.ttf"),
                    ..default()
                },
                TextColor(HUD_COLOUR),
            );

            commands
                .spawn((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        flex_direction: FlexDirection::Column,
                        display: Display::Flex,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    UiTargetCamera(texture_camera),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(150.0),
                            ..default()
                        },
                        ScreenUiElement::AirspeedKts,
                        Text::new("loading..."),
                        text_bundle.clone(),
                    ));
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(150.0),
                            bottom: px(400.0),
                            ..default()
                        },
                        ScreenUiElement::Alpha,
                        Text::new("loading..."),
                        text_bundle.clone(),
                    ));
                    parent.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(150.0),
                            ..default()
                        },
                        ScreenUiElement::Altitude,
                        Text::new("loading..."),
                        text_bundle.clone(),
                    ));
                    parent
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                width: percent(50.0),
                                left: percent(25.0),
                                top: percent(50.0),
                                height: px(2.0),
                                ..default()
                            },
                            BackgroundColor(HUD_COLOUR),
                            ScreenUiElement::Horizon,
                        ))
                        .with_children(|parent| {
                            for i in -10..10 {
                                if i != 0 {
                                    let text = format!("{}", i * -10);
                                    let top = 360.0 * i as f32;
                                    parent.spawn((
                                        Node {
                                            position_type: PositionType::Absolute,
                                            right: percent(22.0),
                                            top: px(top - 20.0),
                                            ..default()
                                        },
                                        Text::new(&text),
                                        text_bundle.clone(),
                                    ));

                                    parent.spawn((
                                        Node {
                                            position_type: PositionType::Absolute,
                                            left: percent(12.0),
                                            top: px(top - 20.0),
                                            ..default()
                                        },
                                        text_bundle.clone(),
                                        Text::new(&text),
                                    ));

                                    for j in 0..3 {
                                        parent.spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                width: px(10.0),
                                                left: percent(35.0 - j as f32 * 5.0),
                                                top: px(top),
                                                height: px(2.0),
                                                ..default()
                                            },
                                            BackgroundColor(HUD_COLOUR),
                                        ));
                                        parent.spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                width: px(10.0),
                                                right: percent(35.0 + j as f32 * 5.0),
                                                top: px(top),
                                                height: px(2.0),
                                                ..default()
                                            },
                                            BackgroundColor(HUD_COLOUR),
                                        ));
                                    }
                                }
                            }
                        });
                });

            let brt = 60.0;
            materials.add(StandardMaterial {
                emissive_texture: Some(image_handle),
                emissive: LinearRgba::new(brt, brt, brt, 1.0),
                base_color: Color::linear_rgba(0.3, 0.15, 0.3, 0.4),
                perceptual_roughness: 0.2,
                alpha_mode: AlphaMode::Premultiplied,
                ..default()
            })
        }

        _ => {
            let label = match screen_data.screen {
                Screens::Left => ScreenUiElement::Throttle,
                Screens::Right => ScreenUiElement::AirspeedKts,
                Screens::Centre => ScreenUiElement::Altitude,
                Screens::Hud => todo!(),
            };

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
                                label,
                                Text::new("loading..."),
                                TextFont {
                                    // font_size: 40.0,
                                    ..default()
                                },
                                TextColor::WHITE,
                            ));
                        });
                });
            // This material has the texture that has been rendered.
            materials.add(StandardMaterial {
                emissive_texture: Some(image_handle),
                emissive: LinearRgba::new(1.0, 1.0, 1.0, 1.0),
                base_color: Color::BLACK,
                perceptual_roughness: 0.2,
                ..default()
            })
        }
    }
}

pub fn update_screens(
    query: Query<(Option<&mut Text>, &ScreenUiElement, &mut UiTransform)>,
    input_axis: Res<InputAxis>,
    vel_tf: Single<(&LinearVelocity, &Transform), With<Aircraft>>,
) {
    let (vel, tf) = *vel_tf;
    for (text, screen, mut ui_transform) in query {
        if let Some(mut text) = text {
            match screen {
                ScreenUiElement::Throttle => {
                    *text = Text::new(format!("{:.2}%", input_axis.throttle * 100.0))
                }
                ScreenUiElement::AirspeedKts => {
                    *text = Text::new(format!("{}", (tf.forward().dot(vel.0) * M_S_TO_KTS) as i32))
                }
                ScreenUiElement::Altitude => {
                    *text = Text::new(format!("{}", (tf.translation.y * METRES_TO_FEET) as i32))
                }
                ScreenUiElement::Alpha => {
                    let velocity_dir = vel_tf.0.to_vec3a().to_vec3();

                    let transform = vel_tf.1;
                    let sin = transform
                        .forward()
                        .cross(velocity_dir)
                        .dot(transform.right().as_vec3());
                    let cos = transform.forward().dot(velocity_dir);
                    let alpha_deg = -sin.atan2(cos).to_degrees();

                    *text = Text::new(format!("AOA {:.2}", alpha_deg))
                }

                _ => *text = Text::new("todo!()"),
            }
        } else {
            if let ScreenUiElement::Horizon = screen {
                let (_, _, roll) = tf.rotation.to_euler(EulerRot::YXZ);
                ui_transform.rotation = Rot2::radians(roll);

                let forward = tf.forward();
                let pitch = forward.y.asin();

                let px_per_rad = 2800.0;
                ui_transform.translation.y = px(pitch * px_per_rad);
                ui_transform.translation.x = px(0.0);
            }
        }
    }
}
