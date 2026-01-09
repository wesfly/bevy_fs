use bevy::prelude::*;

use crate::Aircraft;

#[derive(Component)]
pub struct AltitudeText;

pub fn setup_ui(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(10.0),
            left: px(10.0),
            ..default()
        },
        Text::new("Altitude"),
        AltitudeText,
    ));
}

pub fn update_ui(
    mut altitude: Single<&mut Text, With<AltitudeText>>,
    transform: Single<&Transform, With<Aircraft>>,
) {
    let mut alt_string = String::from("Altitude: ");
    alt_string += &transform.translation.y.round().to_string();
    alt_string += "m";
    altitude.0 = alt_string;
}
