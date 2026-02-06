use crate::{Aircraft, InputAxis};
use bevy::prelude::*;

#[derive(Component)]
struct AltitudeUI;

#[derive(Component)]
struct ThrottleUI;

pub struct UI;
impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app // ok
            .add_systems(Startup, setup_ui)
            .add_systems(Update, (update_altitude, update_throttle));
    }
}

fn setup_ui(mut commands: Commands) {
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
