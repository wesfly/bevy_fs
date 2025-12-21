use crate::{GamepadSettings, InputAxis, IsGamepadConnected, Keymap};
use bevy::{input::gamepad::GamepadConnectionEvent, prelude::*};
pub fn input_system(
    mut is_gamepad_connected: ResMut<IsGamepadConnected>,
    mut input: ResMut<InputAxis>,
    gamepads: Query<(Entity, &Gamepad)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
    gamepad_settings: Res<GamepadSettings>,
    keymap: Res<Keymap>,
) {
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
        if connection_event.connected() == true {
            is_gamepad_connected.0 = true;
        }
    }

    // Switch to gamepad when connected
    if is_gamepad_connected.0 == false {
        button_input_system(input, keyboard_input, keymap);
    } else if is_gamepad_connected.0 == true {
        let gamepad_input = gamepad_input_system(gamepads, connection_events);

        input.x = gamepad_input.0;
        input.y = gamepad_input.1;
        input.z = gamepad_input.2;

        let threshold = gamepad_settings.control_snapping_treshold;
        let threshold_range = -threshold..threshold;

        // Control values snap to zero when in a certain range
        if gamepad_settings.control_snapping_enabled {
            if threshold_range.contains(&gamepad_input.0) {
                input.x = 0.0
            }
            if threshold_range.contains(&gamepad_input.1) {
                input.y = 0.0
            }
            if threshold_range.contains(&gamepad_input.2) {
                input.z = 0.0
            }
        }
    }
}

fn button_input_system(
    mut input: ResMut<'_, InputAxis>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    keymap: Res<'_, Keymap>,
) {
    // X axis (pitch up/down)
    if keyboard_input.pressed(keymap.up) {
        input.x = 1.0;
    } else if keyboard_input.pressed(keymap.down) {
        input.x = -1.0;
    } else {
        input.x = 0.0
    }

    // Z axis (yaw left/right)
    if keyboard_input.pressed(keymap.rudder_left) {
        input.z = -1.0
    } else if keyboard_input.pressed(keymap.rudder_right) {
        input.z = 1.0;
    } else {
        input.z = 0.0
    }

    // Y axis (roll left/right)
    if keyboard_input.pressed(keymap.roll_left) {
        input.y = 1.0
    } else if keyboard_input.pressed(keymap.roll_right) {
        input.y = -1.0;
    } else {
        input.y = 0.0
    }
}

fn gamepad_input_system(
    gamepads: Query<(Entity, &Gamepad)>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
) -> (f32, f32, f32) {
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
    }
    for (_entity, gamepad) in &gamepads {
        let left_stick_x = gamepad.get(GamepadAxis::LeftStickX).unwrap();
        let left_stick_y = gamepad.get(GamepadAxis::LeftStickY).unwrap();
        let right_stick_x = gamepad.get(GamepadAxis::RightStickX).unwrap();

        // Should just use the first gamepad that is connected, having two is rare
        return (left_stick_y, right_stick_x, left_stick_x);
    }

    // return zero if nothing is connected, but this technially shouldn't happen
    return (0.0, 0.0, 0.0);
}
