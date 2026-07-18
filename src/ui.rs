use core::convert::Into;

use crate::{
    ControlInputs, GameState, RunOnceSystemList, Settings,
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    scenery::terrain::Coord,
};
use avian3d::prelude::LinearVelocity;
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    input_focus::{FocusCause, InputFocus},
    picking::hover::HoverMap,
    prelude::*,
    window::WindowMode,
};

#[derive(Component)]
pub struct MenuCamera;

#[derive(Component)]
pub enum UIHudComponent {
    Altitude,
    Throttle,
    Velocity,
}

#[derive(Component, Clone, PartialEq)]
enum SpawnButton {
    AircraftSelector(AircraftTypes),
    Location(Coord),
    Fly,
}

#[derive(Component)]
pub struct Menu;

const FONT_PATH: &str = "fonts/Geist-VariableFont_wght.ttf";

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
        asset_server: Res<AssetServer>,
        mut game_state: ResMut<GameState>,
        scene_query: Query<Entity, With<GlobalTransform>>, // all 3d objects
        ui_hud_query: Query<Entity, With<UIHudComponent>>,
    ) {
        commands.spawn((Camera3d::default(), MenuCamera, IsDefaultUiCamera));

        for entity in &scene_query {
            commands.entity(entity).despawn();
        }

        for entity in &ui_hud_query {
            commands.entity(entity).despawn();
        }

        game_state.in_menu = true;
        let spawn_button = |spawn_button_type: SpawnButton| {
            (
                Button,
                if spawn_button_type.clone() == SpawnButton::Fly {
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: px(5),
                        right: px(5),
                        border: UiRect::all(px(5)),
                        padding: px(32.0).vertical().with_left(px(64)).with_right(px(64)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    }
                } else {
                    Node {
                        border: UiRect::all(px(5)),
                        padding: px(12.0).all(),
                        ..default()
                    }
                },
                spawn_button_type,
                BorderColor::all(Color::WHITE),
                BackgroundColor(Color::BLACK),
            )
        };

        let locations = vec![
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

        let font = asset_server.load(FONT_PATH);

        commands
            .spawn((
                Menu,
                Name::new("Root of Menu"),
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
            ))
            .with_children(|parent| {
                parent.spawn((
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
                            spawn_button(SpawnButton::AircraftSelector(AircraftTypes::Helicopter)),
                            children![(
                                Text::new("Helicopter"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: FontSource::Handle(font.clone()),
                                    ..default()
                                }
                            )]
                        ),
                        (
                            spawn_button(SpawnButton::AircraftSelector(AircraftTypes::Breeze)),
                            children![(
                                Text::new("Breeze C F3"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: FontSource::Handle(font.clone()),
                                    ..default()
                                },
                            ),],
                        ),
                        (
                            spawn_button(SpawnButton::AircraftSelector(AircraftTypes::J3Cub)),
                            children![(
                                Text::new("J-3 Cub"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: FontSource::Handle(font.clone()),
                                    ..default()
                                }
                            )],
                        )
                    ],
                ));

                parent
                    .spawn((
                        Name::new("Location Menu & Fly button"),
                        Node {
                            width: percent(100),
                            height: percent(100),
                            row_gap: px(12.0),
                            column_gap: px(12.0),
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(1, 1.0),
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        let font2 = font.clone(); // hecky borrow checker things
                        parent
                            .spawn((
                                Name::new("Location Menu"),
                                Node {
                                    display: Display::Flex,
                                    flex_direction: FlexDirection::Column,
                                    align_self: AlignSelf::Stretch,
                                    height: percent(100),
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                for (text, location) in locations {
                                    parent.spawn((
                                        spawn_button(SpawnButton::Location(location)),
                                        children![(
                                            Text::new(text.to_string()),
                                            TextColor(Color::WHITE),
                                            TextFont {
                                                font: FontSource::Handle(font2.clone()),
                                                ..default()
                                            },
                                        )],
                                    ));
                                }
                            });

                        parent.spawn((
                            spawn_button(SpawnButton::Fly),
                            children![(
                                Text::new("Fly"),
                                TextColor(Color::WHITE),
                                TextFont {
                                    font: FontSource::Handle(font),
                                    ..default()
                                }
                            ),],
                        ));
                    });
            });
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
            .add_systems(Startup, Menu::spawn)
            .add_systems(
                Update,
                (Self::update_ui_hud, Self::ui_main_loop, Scroll::send_events),
            )
            .add_systems(FixedUpdate, (Self::button_system, toggle_fullscreen))
            .add_observer(Scroll::handler);
    }
}

impl UI {
    fn button_system(
        mut input_focus: ResMut<InputFocus>,
        mut messages: MessageWriter<UIMessage>,
        mut interaction_query: Query<(
            Entity,
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Button,
            &SpawnButton,
        )>,
        mut settings: ResMut<Settings>,
        mut spawn_settings: Local<Option<AircraftTypes>>,
        mut highlighted: Local<(Option<SpawnButton>, Option<SpawnButton>)>, // One field for aircraft sel, one for location sel
    ) {
        for (entity, interaction, mut color, mut border_color, mut button, spawn_button) in
            interaction_query.iter_mut()
        {
            match *interaction {
                Interaction::Pressed => {
                    input_focus.set(entity, FocusCause::Navigated);

                    match spawn_button {
                        SpawnButton::AircraftSelector(AircraftTypes::Breeze) => {
                            *spawn_settings = Some(AircraftTypes::Breeze);
                            highlighted.0 =
                                Some(SpawnButton::AircraftSelector(AircraftTypes::Breeze));
                        }
                        SpawnButton::AircraftSelector(AircraftTypes::Helicopter) => {
                            *spawn_settings = Some(AircraftTypes::Helicopter);
                            highlighted.0 =
                                Some(SpawnButton::AircraftSelector(AircraftTypes::Helicopter));
                        }
                        SpawnButton::AircraftSelector(AircraftTypes::J3Cub) => {
                            *spawn_settings = Some(AircraftTypes::J3Cub);
                            highlighted.0 =
                                Some(SpawnButton::AircraftSelector(AircraftTypes::J3Cub));
                        }

                        SpawnButton::Location(l) => {
                            settings.terrain.coord = l.clone();
                            highlighted.1 = Some(SpawnButton::Location(l.clone()));
                        }
                        SpawnButton::Fly => {
                            if let Some(aircraft) = *spawn_settings {
                                match aircraft {
                                    AircraftTypes::Helicopter => {
                                        messages.write(UIMessage::SpawnHelicopter);
                                    }
                                    AircraftTypes::Breeze => {
                                        messages.write(UIMessage::SpawnBreeze);
                                    }
                                    AircraftTypes::J3Cub => {
                                        messages.write(UIMessage::SpawnJ3Cub);
                                    }
                                }
                                messages.write(UIMessage::SpawnScenery);
                                messages.write(UIMessage::DespawnMenu);
                                messages.write(UIMessage::SpawnUIHud);
                            }
                        }
                    }

                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                }
                Interaction::Hovered => {
                    input_focus.set(entity, FocusCause::Navigated);
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                    *border_color = BorderColor::all(Color::WHITE);
                }
                Interaction::None => {
                    input_focus.clear();
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                    *border_color = BorderColor::all(Color::BLACK);
                }
            }

            if highlighted.0 == Some(spawn_button.clone())
                || highlighted.1 == Some(spawn_button.clone())
            {
                *border_color = Color::srgb(0.45, 0.55, 1.00).into();
            }

            button.set_changed();
        }
    }

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
        transform: Single<&Transform, With<Aircraft>>,
        input: Res<ControlInputs>,
        vel: Single<&LinearVelocity, With<Aircraft>>,
    ) {
        for (mut text, ui_hud_component) in query {
            let forward = match state.aircraft_type {
                AircraftTypes::Helicopter => -transform.local_z(),
                _ => transform.local_x(),
            };
            let string = match ui_hud_component {
                UIHudComponent::Altitude => format!(
                    "Altitude: {}m",
                    &transform.translation.y.round().to_string()
                ),
                UIHudComponent::Throttle => {
                    format!("Throttle: {}%", (input.throttle * 100.0).round())
                }
                UIHudComponent::Velocity => {
                    format!("Velocity: {:?} km/h", (forward.dot(vel.0) * 3.6) as i32)
                }
            };
            text.0 = string;
        }
    }
}

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
#[entity_event(propagate, auto_propagate)]
struct Scroll {
    entity: Entity,
    /// Scroll delta in logical coordinates.
    delta: Vec2,
}

impl Scroll {
    fn handler(
        mut scroll: On<Scroll>,
        mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
    ) {
        let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
            return;
        };

        let max_offset =
            (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

        let delta = &mut scroll.delta;
        if node.overflow.x == OverflowAxis::Scroll && delta.x != 0. {
            // Is this node already scrolled all the way in the direction of the scroll?
            let max = if delta.x > 0. {
                scroll_position.x >= max_offset.x
            } else {
                scroll_position.x <= 0.
            };

            if !max {
                scroll_position.x += delta.x;
                // Consume the X portion of the scroll delta.
                delta.x = 0.;
            }
        }

        if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
            // Is this node already scrolled all the way in the direction of the scroll?
            let max = if delta.y > 0. {
                scroll_position.y >= max_offset.y
            } else {
                scroll_position.y <= 0.
            };

            if !max {
                scroll_position.y += delta.y;
                // Consume the Y portion of the scroll delta.
                delta.y = 0.;
            }
        }

        // Stop propagating when the delta is fully consumed.
        if *delta == Vec2::ZERO {
            scroll.propagate(false);
        }
    }

    fn send_events(
        mut mouse_wheel_reader: MessageReader<MouseWheel>,
        hover_map: Res<HoverMap>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut commands: Commands,
    ) {
        for mouse_wheel in mouse_wheel_reader.read() {
            let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

            if mouse_wheel.unit == MouseScrollUnit::Line {
                delta *= 10.0;
            }

            if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
                std::mem::swap(&mut delta.x, &mut delta.y);
            }

            for pointer_map in hover_map.values() {
                for entity in pointer_map.keys().copied() {
                    commands.trigger(Scroll { entity, delta });
                }
            }
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
