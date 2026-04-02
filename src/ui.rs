use crate::{
    InputAxis, RunOnceSystemList,
    aircraft::{Aircraft, AircraftTypes},
    scenery::terrain::Coordinates,
};
use avian3d::prelude::LinearVelocity;
use bevy::{input_focus::InputFocus, prelude::*, window::WindowMode};

#[derive(Component)]
pub struct MenuCamera;

#[derive(Component)]
enum UIHudComponent {
    Altitude,
    Throttle,
    Velocity,
}

#[derive(Component, Clone, PartialEq)]
enum SpawnButton {
    AircraftSelector(AircraftTypes),
    Fly,
}

#[derive(Component)]
struct Menu;

const FONT_PATH: &str = "fonts/Geist-VariableFont_wght.ttf";

impl Menu {
    fn despawn(commands: &mut Commands, menu_query: Query<Entity, With<Menu>>) {
        for entity in menu_query {
            commands.entity(entity).despawn();
        }
    }

    fn spawn(mut commands: Commands, asset_server: Res<AssetServer>) {
        let aircraft_spawn_button = |spawn_button_type: SpawnButton| {
            (
                Button,
                spawn_button_type,
                Menu,
                Node {
                    border: UiRect::all(px(5)),
                    padding: px(12.0).all(),
                    ..default()
                },
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::BLACK),
            )
        };

        commands.spawn((
            Menu,
            Node {
                // fill the entire window
                width: percent(100),
                height: percent(100),
                padding: px(12.0).all(),
                row_gap: px(12.0),
                column_gap: px(12.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(2, 1.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![
                (
                    Name::new("Aircraft Menu"),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        row_gap: px(12.0),
                        column_gap: px(12.0),
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(1, 1.0),
                        ..default()
                    },
                    children![
                        (
                            aircraft_spawn_button(SpawnButton::AircraftSelector(
                                AircraftTypes::Helicopter
                            )),
                            children![(
                                Text::new("Spawn Helicopter"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: asset_server.load(FONT_PATH),
                                    ..default()
                                }
                            )]
                        ),
                        (
                            aircraft_spawn_button(SpawnButton::AircraftSelector(
                                AircraftTypes::Aeroplane
                            )),
                            children![(
                                Text::new("Spawn Aeroplane"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: asset_server.load(FONT_PATH),
                                    ..default()
                                }
                            )],
                        )
                    ]
                ),
                (
                    aircraft_spawn_button(SpawnButton::Fly),
                    children![(
                        Text::new("Fly"),
                        TextColor(Color::WHITE),
                        TextFont {
                            font: asset_server.load(FONT_PATH),
                            ..default()
                        }
                    )],
                )
            ],
        ));
    }
}

#[derive(Message)]
enum UIMessage {
    DespawnMenu,
    SpawnUIHud,
    SpawnScenery,
    SpawnHelicopter,
    SpawnAeroplane,
}

pub struct UI;
impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (Self::ui_main_setup, Menu::spawn))
            .init_resource::<InputFocus>()
            .add_message::<UIMessage>()
            .add_systems(
                Update,
                (
                    Self::ui_main_loop,
                    Self::update_ui_hud,
                    Self::button_system,
                    toggle_fullscreen,
                ),
            );
    }
}

struct SpawnSettings {
    aircraft: Option<AircraftTypes>,
    location: Option<Coordinates>,
}

impl Default for SpawnSettings {
    fn default() -> Self {
        SpawnSettings {
            aircraft: None,
            location: None,
        }
    }
}

impl UI {
    fn button_system(
        mut input_focus: ResMut<InputFocus>,
        mut messages: MessageWriter<UIMessage>,
        mut interaction_query: Query<
            (
                Entity,
                &Interaction,
                &mut BackgroundColor,
                &mut BorderColor,
                &mut Button,
                &SpawnButton,
            ),
            Changed<Interaction>,
        >,
        mut spawn_settings: Local<SpawnSettings>,
        mut highlighted: Local<Option<SpawnButton>>,
    ) {
        for (entity, interaction, mut color, mut border_color, mut button, spawn_button) in
            interaction_query.iter_mut()
        {
            match *interaction {
                Interaction::Pressed => {
                    input_focus.set(entity);

                    match spawn_button {
                        SpawnButton::AircraftSelector(AircraftTypes::Aeroplane) => {
                            spawn_settings.aircraft = Some(AircraftTypes::Aeroplane);
                            *highlighted =
                                Some(SpawnButton::AircraftSelector(AircraftTypes::Aeroplane));
                        }
                        SpawnButton::AircraftSelector(AircraftTypes::Helicopter) => {
                            spawn_settings.aircraft = Some(AircraftTypes::Helicopter);
                            *highlighted =
                                Some(SpawnButton::AircraftSelector(AircraftTypes::Helicopter));
                        }
                        SpawnButton::Fly => {
                            if let Some(aircraft) = &spawn_settings.aircraft {
                                match aircraft {
                                    AircraftTypes::Helicopter => {
                                        messages.write(UIMessage::SpawnHelicopter);
                                    }
                                    AircraftTypes::Aeroplane => {
                                        messages.write(UIMessage::SpawnAeroplane);
                                    }
                                }
                                messages.write(UIMessage::SpawnScenery);
                                messages.write(UIMessage::DespawnMenu);
                                messages.write(UIMessage::SpawnUIHud);
                            }
                        }
                    }

                    *color = Color::srgb(0.15, 0.15, 0.15).into();

                    button.set_changed();
                }
                Interaction::Hovered => {
                    input_focus.set(entity);
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                    *border_color = BorderColor::all(Color::WHITE);
                    button.set_changed();
                }
                Interaction::None => {
                    input_focus.clear();
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                    *border_color = BorderColor::all(Color::BLACK);
                }
            }
            if *highlighted == Some(spawn_button.clone()) {
                *border_color = Color::srgb(0.45, 0.55, 1.00).into();
            }
        }
    }

    fn ui_main_loop(
        mut commands: Commands,
        mut messages: MessageReader<UIMessage>,
        menu_query: Query<Entity, With<Menu>>,
        systems: Res<RunOnceSystemList>,
        system: Res<RunOnceSystemList>,
    ) {
        for message in messages.read() {
            match message {
                UIMessage::DespawnMenu => Menu::despawn(&mut commands, menu_query),
                UIMessage::SpawnUIHud => commands.run_system(systems.0["spawn_ui_hud"]),
                UIMessage::SpawnScenery => {
                    commands.run_system(system.0["setup_scene"]);
                }
                UIMessage::SpawnHelicopter => {
                    commands.run_system(system.0["spawn_helicopter"]);
                }
                UIMessage::SpawnAeroplane => {
                    commands.run_system(system.0["setup_aeroplane"]);
                }
            }
        }
    }

    fn ui_main_setup(mut commands: Commands) {
        commands.spawn((Camera3d::default(), MenuCamera));
    }

    pub fn setup_ui_hud(mut commands: Commands) {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(10.0),
                left: px(10.0),
                ..default()
            },
            Text::new("Altitude"),
            UIHudComponent::Altitude,
        ));
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(10.0),
                right: px(10.0),
                ..default()
            },
            Text::new("Throttle"),
            UIHudComponent::Throttle,
        ));

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(50.0),
                right: px(10.0),
                ..default()
            },
            Text::new("Velocity"),
            UIHudComponent::Velocity,
        ));
    }

    fn update_ui_hud(
        query: Query<(&mut Text, &UIHudComponent)>,
        transform: Single<&Transform, With<Aircraft>>,
        input: Res<InputAxis>,
        vel: Single<&LinearVelocity, With<Aircraft>>,
    ) {
        for (mut text, ui_hud_component) in query {
            let string = match ui_hud_component {
                UIHudComponent::Altitude => format!(
                    "Altitude: {}m",
                    &transform.translation.y.round().to_string()
                ),
                UIHudComponent::Throttle => {
                    format!("Throttle: {}%", (input.throttle * 100.0).round())
                }
                UIHudComponent::Velocity => format!(
                    "Velocity: {:?} km/h",
                    (transform.forward().dot(vel.0) * 3.6) as i32
                ),
            };
            text.0 = string;
        }
    }
}

fn toggle_fullscreen(input: Res<ButtonInput<KeyCode>>, mut windows: Query<&mut Window>) {
    if input.just_pressed(KeyCode::F11) {
        let mut window = windows.single_mut().unwrap();

        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }
}
