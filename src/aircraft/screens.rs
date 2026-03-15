use bevy::{
    asset::RenderAssetUsages,
    camera::RenderTarget,
    color::palettes::css::{BLACK, BLUE},
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Component)]
pub enum Screens {
    Left,
    Right,
    Centre,
}

#[derive(Deserialize, Debug)]
pub struct Screen {
    pub screen: Screens,
}

pub fn spawn_texture_ui_whatever(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    images: &mut ResMut<Assets<Image>>,
    screen_data: &Screen,
) -> Handle<StandardMaterial> {
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
    materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        ..default()
    })
}
