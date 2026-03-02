use crate::{
    GameState, InputAxis, RunOnceSystemList,
    aircraft::{Aircraft, AircraftState},
};
use avian3d::prelude::LinearVelocity;
use bevy::{input_focus::InputFocus, prelude::*};

#[derive(Component)]
pub struct MenuCamera;

#[derive(Component)]
struct AltitudeUI;

#[derive(Component)]
struct ThrottleUI;

#[derive(Component)]
struct VelocityUI;

#[derive(Component)]
enum SpawnButton {
    Aeroplane,
    Helicopter,
}

#[derive(Component)]
struct Menu;

#[derive(Message)]
struct UIMessage {
    despawn: bool,
}

pub struct UI;
impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .init_resource::<InputFocus>()
            .add_message::<UIMessage>()
            .add_systems(
                Update,
                (
                    update_altitude,
                    update_velocity,
                    update_throttle,
                    button_system,
                    delete_menu,
                ),
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
        Node {
            position_type: PositionType::Absolute,
            bottom: px(50.0),
            right: px(10.0),
            ..default()
        },
        Text::new("Velocity"),
        VelocityUI,
    ));

    commands.spawn((
        Button,
        SpawnButton::Aeroplane,
        Menu,
        Node {
            position_type: PositionType::Absolute,
            top: px(200.0),
            right: px(10.0),
            width: Val::Percent(40.0),
            height: percent(40.0),
            padding: UiRect::all(px(10.0)),
            border: UiRect::all(px(5)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(Color::BLACK),
        children![(
            Text::new("Spawn Aeroplane"),
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        )],
    ));
    commands.spawn((
        Button,
        SpawnButton::Helicopter,
        Menu,
        Node {
            position_type: PositionType::Absolute,
            top: px(200.0),
            left: px(10.0),
            width: Val::Percent(40.0),
            height: percent(40.0),
            padding: UiRect::all(px(10.0)),
            border: UiRect::all(px(5)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(Color::WHITE),
        BackgroundColor(Color::BLACK),
        children![(
            Text::new("Spawn Helicopter"),
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        )],
    ));
}

fn button_system(
    mut commands: Commands,
    system: Res<RunOnceSystemList>,
    mut game_state: ResMut<GameState>,
    mut input_focus: ResMut<InputFocus>,
    mut state: ResMut<AircraftState>,
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
) {
    for (entity, interaction, mut color, mut border_color, mut button, spawn_button) in
        interaction_query.iter_mut()
    {
        if *game_state == GameState::Menu {
            match *interaction {
                Interaction::Pressed => {
                    input_focus.set(entity);
                    *color = Color::srgb(0.15, 0.15, 0.15).into();
                    button.set_changed();

                    *game_state = GameState::Running;
                    match spawn_button {
                        SpawnButton::Aeroplane => {
                            state.aircraft_type = crate::aircraft::AircraftTypes::Aeroplane;

                            commands.run_system(system.0["setup_aeroplane"]);
                        }
                        SpawnButton::Helicopter => {
                            state.aircraft_type = crate::aircraft::AircraftTypes::Helicopter;

                            commands.run_system(system.0["setup_helicopter"]);
                        }
                    }
                    commands.run_system(system.0["setup_scene"]);
                    commands.run_system(system.0["setup_terrain"]);
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
        } else {
            messages.write(UIMessage { despawn: true });
        }
    }
}

fn delete_menu(
    mut messages: MessageReader<UIMessage>,
    mut commands: Commands,
    menu_query: Query<Entity, With<Menu>>,
) {
    for message in messages.read() {
        if message.despawn {
            for entity in menu_query {
                commands.entity(entity).despawn();
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

fn update_velocity(
    vel: Single<&LinearVelocity, With<Aircraft>>,
    transform: Single<&Transform, With<Aircraft>>,
    mut velocity: Single<&mut Text, With<VelocityUI>>,
) {
    let string = format!(
        "Velocity: {:?} km/h",
        (transform.forward().dot(vel.0) * 3.6) as i32
    );
    velocity.0 = string;
}
