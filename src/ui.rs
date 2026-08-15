use crate::{
    ControlInputs, EARTH_RADIUS, GameState, RunOnceSystemList,
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    scenery::terrain::Coord,
};
use avian3d::prelude::LinearVelocity;
use bevy::{
    feathers::{FeathersPlugins, containers::*, controls::*, theme::ThemedText},
    input_focus::InputFocus,
    prelude::*,
    ui::Selected,
    ui_widgets::{Activate, listbox_update_selection},
    window::WindowMode,
};
use core::convert::Into;

#[derive(Component)]
pub struct MenuCamera;

#[derive(Component)]
pub enum UIHudComponent {
    Altitude,
    Throttle,
    Velocity,
}

#[derive(Component, Clone, Debug, Default)]
enum AircraftSelector {
    #[default]
    Breeze,
    Helicopter,
    J3Cub,
}

impl AircraftSelector {
    pub const BREEZE: Self = Self::Breeze;
    pub const J3CUB: Self = Self::J3Cub;
    pub const HELICOPTER: Self = Self::Helicopter;
}

#[derive(Component, Clone, Default)]
struct LocationSelector(Coord);

#[derive(Component, Default, Clone)]
pub struct Menu;

// const FONT_PATH: &str = "fonts/Geist-VariableFont_wght.ttf";

fn aircraft_selector() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            padding: px(8),
            row_gap: px(8),
            width: percent(30),
            min_width: px(200),
        }
        Children[
            subpane() Children [
                subpane_header() Children [
                    (Text("Aircraft") ThemedText),
                ],
                subpane_body() Children [
                    @FeathersListView {
                        @rows: { bsn_list! [
                            @FeathersListRow AircraftSelector::BREEZE Children [(
                                Text("Breeze")
                                ThemedText
                                Node {
                                    height: px(100)
                                }
                            )],
                            @FeathersListRow AircraftSelector::J3CUB Children [(
                                Text("J-3 Cub")
                                ThemedText
                                Node {
                                    height: px(100)
                                }
                            )],
                            @FeathersListRow AircraftSelector::HELICOPTER Children [(
                                Text("Helicopter")
                                ThemedText
                                Node {
                                    height: px(100)
                                }
                            )],
                        ]}
                    }
                    Node {
                        min_height: px(100)
                    }
                    on(listbox_update_selection)

                ]
            ]
        ]
    }
}

fn location_menu() -> impl Scene {
    let locations: Vec<(&str, Coord)> = vec![
        (
            "Toulon, FR",
            Coord {
                lat: 43.12694,
                long: 5.93071,
            },
        ),
        (
            "Hobart, AU",
            Coord {
                lat: -42.88369,
                long: 147.3287,
            },
        ),
        (
            "San Francisco, USA",
            Coord {
                lat: 37.7922,
                long: -122.4385,
            },
        ),
        (
            "Cape Town, ZA",
            Coord {
                lat: -33.9114,
                long: 18.5033,
            },
        ),
        (
            "Vancouver, CA",
            Coord {
                lat: 49.2920,
                long: -123.1416,
            },
        ),
        (
            "London, UK",
            Coord {
                lat: 51.5074,
                long: -0.1278,
            },
        ),
        (
            "Tokyo, JP",
            Coord {
                lat: 35.6762,
                long: 139.6503,
            },
        ),
        (
            "New York, US",
            Coord {
                lat: 40.7128,
                long: -74.0060,
            },
        ),
        (
            "Sydney, AU",
            Coord {
                lat: -33.8688,
                long: 151.2093,
            },
        ),
        (
            "Paris, FR",
            Coord {
                lat: 48.8566,
                long: 2.3522,
            },
        ),
        (
            "Cairo, EG",
            Coord {
                lat: 30.0444,
                long: 31.2357,
            },
        ),
        (
            "Rio de Janeiro, BR",
            Coord {
                lat: -22.9068,
                long: -43.1729,
            },
        ),
        (
            "Berlin, DE",
            Coord {
                lat: 52.5200,
                long: 13.4050,
            },
        ),
        (
            "Mumbai, IN",
            Coord {
                lat: 19.0760,
                long: 72.8777,
            },
        ),
        (
            "Zürich, CH",
            Coord {
                lat: 47.37445,
                long: 8.541039,
            },
        ),
        (
            "Lucerne, CH",
            Coord {
                lat: 47.052099,
                long: 8.30899,
            },
        ),
        (
            "Nürnberg, DE",
            Coord {
                lat: 49.45387,
                long: 11.0773,
            },
        ),
        (
            "Guăngzhōu Shì, CN",
            Coord {
                lat: 23.128864,
                long: 113.259009,
            },
        ),
    ];

    let rows: Vec<_> = locations
        .into_iter()
        .map(|(name, coord)| {
            bsn! {
                @FeathersListRow
                LocationSelector(coord)
                Children [(
                    ThemedText
                    Text({name.to_string()})
                )]
            }
        })
        .collect();

    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            justify_content: JustifyContent::Start,
            padding: px(8),
            row_gap: px(8),
            width: percent(30),
            min_width: px(200),
            position_type: PositionType::Absolute,
            top: px(10),
            right: px(10),
            max_height: percent(100),
            overflow: Overflow::clip_y(),
        }

        Children[
            subpane() Children [
                subpane_header() Children [
                    (Text("Location") ThemedText),
                ],
                subpane_body() Children [
                    @FeathersListView {
                        @rows: { Box::new(rows) as Box<dyn SceneList> }
                    }
                    Node {
                        max_height: vh(75)
                    }
                    on(listbox_update_selection)

                ]
            ]
        ]
    }
}

fn ui_root() -> impl Scene {
    bsn! {
        Menu
        Node {
            height: percent(100.0),
            width: percent(100.0)
        }
        Children [
            aircraft_selector(),
            fly_button(),
            location_menu(),
        ]
    }
}

fn fly_button() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10.0),
            right: px(10.0),
        }
        @FeathersButton {
            @caption: bsn! {
                Text("Fly")
                ZIndex(1)
                ThemedText
                TextFont {
                    font_size: FontSize::Px(54.0)
                }
                Node {
                    padding: UiRect::horizontal(px(50)) // TODO: vertical axis doesn't work?
                }
            },
            @variant: ButtonVariant::Primary,
        }
        on(|
            _activate: On<Activate>,
            mut messages: MessageWriter<UIMessage>,
            aircraft_selector: Option<Single<&AircraftSelector, With<Selected>>>,
            location_selector: Option<Single<&LocationSelector, With<Selected>>>,
            mut settings: ResMut<crate::Settings>,
       | {
            if !aircraft_selector.is_some() {
                error!("No aircraft selected");
                return;
            }

            if location_selector.is_some() {
                settings.terrain.coord = location_selector.unwrap().0;
            }

            messages.write(UIMessage::SpawnScenery);
            messages.write(UIMessage::DespawnMenu);
            messages.write(UIMessage::SpawnUIHud);
            match **aircraft_selector.unwrap() {
                AircraftSelector::Breeze => {
                    messages.write(UIMessage::SpawnBreeze);
                }
                AircraftSelector::J3Cub => {
                    messages.write(UIMessage::SpawnJ3Cub);
                }
                AircraftSelector::Helicopter => {
                    messages.write(UIMessage::SpawnHelicopter);
                }
            }
        })
    }
}

impl Menu {
    fn despawn(
        commands: &mut Commands,
        menu_query: Query<Entity, With<Menu>>,
        game_state: &mut ResMut<GameState>,
    ) {
        game_state.in_menu = true;
        for entity in menu_query {
            commands.entity(entity).despawn();
        }
    }

    pub fn spawn(
        mut commands: Commands,
        mut game_state: ResMut<GameState>,
        scene_query: Query<Entity, With<GlobalTransform>>, // all 3d objects
        ui_hud_query: Query<Entity, With<UIHudComponent>>,
    ) {
        commands.spawn((Camera2d::default(), MenuCamera, IsDefaultUiCamera));

        for entity in &scene_query {
            commands.entity(entity).despawn();
        }

        for entity in &ui_hud_query {
            commands.entity(entity).despawn();
        }

        game_state.in_menu = true;

        commands.spawn_scene(ui_root());
    }
}

#[derive(Message)]
pub enum UIMessage {
    SpawnMenu,
    DespawnMenu,
    SpawnUIHud,
    SpawnScenery,
    SpawnHelicopter,
    SpawnBreeze,
    SpawnJ3Cub,
}

pub struct UI;
impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFocus>()
            .add_message::<UIMessage>()
            .add_plugins(FeathersPlugins)
            .add_systems(Startup, Menu::spawn)
            .add_systems(Update, (Self::update_ui_hud, Self::ui_main_loop))
            .add_systems(FixedUpdate, toggle_fullscreen);
    }
}

impl UI {
    fn ui_main_loop(
        mut commands: Commands,
        mut messages: MessageReader<UIMessage>,
        menu_query: Query<Entity, With<Menu>>,
        systems: Res<RunOnceSystemList>,
        mut game_state: ResMut<GameState>,
    ) {
        for message in messages.read() {
            match message {
                UIMessage::DespawnMenu => Menu::despawn(&mut commands, menu_query, &mut game_state),
                UIMessage::SpawnMenu => {
                    commands.run_system(systems.0["spawn_menu"]);
                }
                UIMessage::SpawnUIHud => commands.run_system(systems.0["spawn_ui_hud"]),
                UIMessage::SpawnScenery => {
                    commands.run_system(systems.0["setup_scene"]);
                }
                UIMessage::SpawnHelicopter => {
                    commands.run_system(systems.0["spawn_helicopter"]);
                }
                UIMessage::SpawnBreeze => {
                    commands.run_system(systems.0["setup_breeze"]);
                }
                UIMessage::SpawnJ3Cub => {
                    commands.run_system(systems.0["setup_j3cub"]);
                }
            }
        }
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
        state: Res<AircraftState>,
        aircraft: Single<(&Transform, &avian3d::prelude::Position), With<Aircraft>>,
        input: Res<ControlInputs>,
        vel: Single<&LinearVelocity, With<Aircraft>>,
    ) {
        let (transform, position) = *aircraft;
        for (mut text, ui_hud_component) in query {
            let forward = match state.aircraft_type {
                AircraftTypes::Helicopter => -transform.local_z(),
                _ => transform.local_x(),
            };
            let string = match ui_hud_component {
                UIHudComponent::Altitude => {
                    format!(
                        "Altitude: {}m",
                        (position.length() - EARTH_RADIUS as f64)
                            .round()
                            .to_string()
                    )
                }
                UIHudComponent::Throttle => {
                    format!("Throttle: {}%", (input.throttle * 100.0).round())
                }
                UIHudComponent::Velocity => {
                    format!(
                        "Velocity: {:?} km/h",
                        (forward.dot(vel.0.as_vec3()) * 3.6) as i32
                    )
                }
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
