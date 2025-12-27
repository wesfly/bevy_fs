use crate::{GamepadSettings, InputAxis, IsGamepadConnected, Keymap};
use bevy::{input::gamepad::GamepadConnectionEvent, prelude::*};
pub fn input_system(
    mut is_gamepad_connected: ResMut<IsGamepadConnected>,
    mut input: ResMut<InputAxis>,
    gamepad: Single<(Entity, &Gamepad)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut connection_events: MessageReader<GamepadConnectionEvent>,
    gamepad_settings: Res<GamepadSettings>,
    keymap: Res<Keymap>,
) {
    info!("calling input_system");
    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
        if connection_event.connected() == true {
            is_gamepad_connected.0 = true;
            info!("Gamepad connected.")
        }
    }

    info!("{}", is_gamepad_connected.0);

    // Switch to gamepad when connected
    if is_gamepad_connected.0 == false {
        info!("yeeeh");
        button_input_system(input, keyboard_input, keymap);
    } else if is_gamepad_connected.0 == true {
        let gamepad_input = gamepad_input_system(gamepad, connection_events);

        input.x = gamepad_input.0; // pitch
        input.y = gamepad_input.1; // roll
        input.z = gamepad_input.2; // yaw

        let threshold = gamepad_settings.control_snapping_treshold;
        let threshold_range = -threshold..threshold;

        // Control values snap to zero when in a certain range
        if gamepad_settings.control_snapping_enabled {
            if threshold_range.contains(&gamepad_input.0) {
                input.x = 0.
            }
            if threshold_range.contains(&gamepad_input.1) {
                input.y = 0.
            }
            // if threshold_range.contains(&gamepad_input.2) {
            //     input.z = 0.
            // }
        }
        if keyboard_input.pressed(keymap.throttle_up) {
            input.w += 0.01;
        }
        if keyboard_input.pressed(keymap.throttle_down) {
            input.w += -0.01
        }

        input.w = input.w.clamp(-1., 1.);
    }
}

fn button_input_system(
    mut input: ResMut<'_, InputAxis>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    keymap: Res<'_, Keymap>,
) {
    info!("ts now called");
    // X axis (pitch up/down)
    if keyboard_input.pressed(keymap.up) {
        input.x = 1.;
    } else if keyboard_input.pressed(keymap.down) {
        input.x = -1.;
    } else {
        input.x = 0.
    }

    // Z axis (yaw left/right)
    if keyboard_input.pressed(keymap.rudder_left) {
        input.z = -1.
    } else if keyboard_input.pressed(keymap.rudder_right) {
        input.z = 1.;
    } else {
        input.z = 0.
    }

    // Y axis (roll left/right)
    if keyboard_input.pressed(keymap.roll_left) {
        input.y = 1.
    } else if keyboard_input.pressed(keymap.roll_right) {
        input.y = -1.;
    } else {
        input.y = 0.
    }

    if keyboard_input.pressed(keymap.throttle_up) {
        input.w += 0.01;
    }
    if keyboard_input.pressed(keymap.throttle_down) {
        input.w += -0.01
    }

    info!("Using button_input_system.");
}

fn gamepad_input_system(
    gamepad: Single<(Entity, &Gamepad)>, // I won't handle multiple gamepad for simplicity
    mut connection_events: MessageReader<GamepadConnectionEvent>,
) -> (f32, f32, f32, f32) {
    info!("Using gamepad.");

    for connection_event in connection_events.read() {
        info!("{:?}", connection_event);
    }
    let left_stick_x = gamepad.1.get(GamepadAxis::LeftStickX).unwrap();
    let left_stick_y = gamepad.1.get(GamepadAxis::LeftStickY).unwrap();
    let right_stick_y = 1.;

    let mut right_stick_x = 0.; // The right side of the stick doesn't work, but this can't be zero, so I do it manually
    if gamepad.1.get(GamepadAxis::RightStickX).unwrap() == 0. {
        if gamepad.1.get(GamepadButton::DPadLeft).unwrap() > 0.5 {
            right_stick_x = 1.;
        }
        if gamepad.1.get(GamepadButton::DPadRight).unwrap() > 0.5 {
            right_stick_x = -1.;
        }
    } else {
        right_stick_x = gamepad.1.get(GamepadAxis::RightStickX).unwrap();
        #[cfg(debug_assertions)]
        warn!("this axis works now??")
    }

    // pitch, roll, yaw, throttle
    return (left_stick_y, left_stick_x, right_stick_x, right_stick_y);
}
