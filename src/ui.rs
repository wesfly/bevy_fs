use crate::{Aircraft, GameState, InputAxis, Settings, sse, terrain::TerrainData};
use bevy::{
    input_focus::InputFocus,
    pbr::{ExtendedMaterial, ScatteringMedium},
    prelude::*,
};

#[derive(Message)]
struct GameModeChanged(GameState);

#[derive(Component)]
struct MenuCamera;

#[derive(Component)]
struct AltitudeUI;

#[derive(Component)]
struct ThrottleUI;

pub struct UI;
impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .init_resource::<InputFocus>()
            .add_message::<GameModeChanged>()
            .add_systems(
                Update,
                (update_altitude, update_throttle, button_system, spawn_scene),
            );
    }
}

fn setup_ui(mut commands: Commands) {
    commands.spawn((Camera3d::default(), MenuCamera));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10.0),
            left: px(10.0),
            ..default()
        },
        Text::new("Altitude"),
        AltitudeUI,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10.0),
            right: px(10.0),
            ..default()
        },
        Text::new("Throttle"),
        ThrottleUI,
    ));

    commands.spawn((
        Button,
        Node {
            position_type: PositionType::Absolute,
            top: px(110.0),
            left: px(10.0),
            width: px(150),
            height: px(65),
            border: UiRect::all(px(5)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            // border_radius: BorderRadius::MAX,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(Color::BLACK),
        children![(
            Text::new("Spawn"),
            TextFont {
                font_size: 33.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        )],
    ));
}

fn button_system(
    mut game_state: ResMut<GameState>,
    mut input_focus: ResMut<InputFocus>,
    mut messages: MessageWriter<GameModeChanged>,
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &Children,
        ),
        Changed<Interaction>,
    >,
    mut text_query: Query<&mut Text>,
) {
    for (entity, interaction, mut color, mut border_color, mut button, children) in
        interaction_query.iter_mut()
    {
        let mut text = text_query.get_mut(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = BackgroundColor::DEFAULT;
                button.set_changed();
                if *game_state == GameState::Menu {
                    *game_state = GameState::Running;
                    messages.write(GameModeChanged(GameState::Running));
                }
            }
            Interaction::Hovered => {
                input_focus.set(entity);
                // *color = HOVERED_BUTTON.into();
                *border_color = BorderColor::all(Color::WHITE);
                button.set_changed();
            }
            Interaction::None => {
                input_focus.clear();
                **text = "Spawn".to_string();
                *color = Color::srgb(0.15, 0.15, 0.15).into();
                *border_color = BorderColor::all(Color::BLACK);
            }
        }
    }
}

fn update_altitude(
    mut altitude: Single<&mut Text, With<AltitudeUI>>,
    transform: Single<&Transform, With<Aircraft>>,
) {
    let string = format!(
        "Altitude: {}m",
        &transform.translation.y.round().to_string()
    );
    altitude.0 = string;
}

fn update_throttle(input: Res<InputAxis>, mut throttle: Single<&mut Text, With<ThrottleUI>>) {
    let string = format!("Throttle: {}%", (input.throttle * 100.0).round());
    throttle.0 = string;
}

fn spawn_scene(
    mut commands: Commands,
    camera: Single<Entity, With<MenuCamera>>,
    asset_server: Res<AssetServer>,
    graphs: ResMut<Assets<AnimationGraph>>,
    settings: Res<Settings>,
    meshes: ResMut<Assets<Mesh>>,
    water_materials: Option<ResMut<Assets<ExtendedMaterial<StandardMaterial, sse::Water>>>>,
    scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    materials: ResMut<Assets<StandardMaterial>>,
    terrain_data: ResMut<TerrainData>,
    mut messages: MessageReader<GameModeChanged>,
) {
    match messages.read().last() {
        Some(GameModeChanged(GameState::Running)) => {
            commands.entity(*camera).despawn();
            crate::setup_scene(
                commands,
                asset_server,
                graphs,
                settings,
                meshes,
                water_materials,
                scattering_mediums,
                materials,
                terrain_data,
            );
        }
        _ => {}
    }
}
