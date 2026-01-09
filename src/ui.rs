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
    let alt_string = format!(
        "Altitude: {}m",
        &transform.translation.y.round().to_string()
    );
    altitude.0 = alt_string;
}
