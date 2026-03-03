use crate::{
    InputAxis, RunOnceSystemList,
    aircraft::{Aircraft, AircraftState},
};
use avian3d::prelude::LinearVelocity;
use bevy::{input_focus::InputFocus, prelude::*};

#[derive(Component)]
pub struct MenuCamera;

#[derive(Component)]
enum UIHudComponent {
    Altitude,
    Throttle,
    Velocity,
}

#[derive(Component)]
enum SpawnButton {
    Aeroplane,
    Helicopter,
}

#[derive(Component)]
struct Menu;

impl Menu {
    fn menu(commands: &mut Commands, menu_query: Query<Entity, With<Menu>>) {
        for entity in menu_query {
            commands.entity(entity).despawn();
        }
    }

    fn spawn(mut commands: Commands) {
        commands.spawn((
            Button,
            SpawnButton::Aeroplane,
            Menu,
            Node {
                position_type: PositionType::Absolute,
                top: px(200.0),
                right: px(10.0),
                width: percent(40.0),
                height: percent(40.0),
                padding: UiRect::all(px(10.0)),
                border: UiRect::all(px(5)),
                justify_content: JustifyContent::Center,
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
                width: percent(40.0),
                height: percent(40.0),
                padding: UiRect::all(px(10.0)),
                border: UiRect::all(px(5)),
                justify_content: JustifyContent::Center,
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
}

#[derive(Message)]
enum UIMessage {
    DespawnMenu,
}

pub struct UI;
impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (ui_main_setup, Menu::spawn))
            .init_resource::<InputFocus>()
            .add_message::<UIMessage>()
            .add_systems(Update, (ui_main_loop, update_ui_hud, button_system));
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

fn button_system(
    mut commands: Commands,
    system: Res<RunOnceSystemList>,
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
        match *interaction {
            Interaction::Pressed => {
                input_focus.set(entity);
                *color = Color::srgb(0.15, 0.15, 0.15).into();
                button.set_changed();

                match spawn_button {
                    SpawnButton::Aeroplane => {
                        state.aircraft_type = crate::aircraft::AircraftTypes::Aeroplane;

                        commands.run_system(system.0["setup_aeroplane"]);
                    }
                    SpawnButton::Helicopter => {
                        state.aircraft_type = crate::aircraft::AircraftTypes::Helicopter;

                        commands.run_system(system.0["spawn_helicopter"]);
                    }
                }
                commands.run_system(system.0["setup_scene"]);
                commands.run_system(system.0["setup_terrain"]);

                messages.write(UIMessage::DespawnMenu);
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
    }
}

fn ui_main_loop(
    mut commands: Commands,
    mut messages: MessageReader<UIMessage>,
    menu_query: Query<Entity, With<Menu>>,
    systems: Res<RunOnceSystemList>,
) {
    for message in messages.read() {
        match message {
            UIMessage::DespawnMenu => {
                Menu::menu(&mut commands, menu_query);
                commands.run_system(systems.0["spawn_ui_hud"]);
            }
        }
    }
}

fn ui_main_setup(mut commands: Commands) {
    commands.spawn((Camera3d::default(), MenuCamera));
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
            UIHudComponent::Throttle => format!("Throttle: {}%", (input.throttle * 100.0).round()),
            UIHudComponent::Velocity => format!(
                "Velocity: {:?} km/h",
                (transform.forward().dot(vel.0) * 3.6) as i32
            ),
        };
        text.0 = string;
    }
}
