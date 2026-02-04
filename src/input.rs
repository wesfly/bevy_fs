use crate::{CameraSettings, InputAxis, Settings};
use bevy::prelude::*;

#[derive(Resource)]
pub struct Keymap {
    pub reset_camera: KeyCode,
    up: KeyCode,
    down: KeyCode,
    rudder_left: KeyCode,
    rudder_right: KeyCode,
    roll_left: KeyCode,
    roll_right: KeyCode,
    throttle_up: KeyCode,
    throttle_down: KeyCode,
    change_camera: KeyCode,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            reset_camera: KeyCode::KeyR,
            up: KeyCode::KeyW,
            down: KeyCode::KeyS,
            rudder_left: KeyCode::KeyQ,
            rudder_right: KeyCode::KeyE,
            roll_left: KeyCode::KeyA,
            roll_right: KeyCode::KeyD,
            throttle_up: KeyCode::PageUp,
            throttle_down: KeyCode::PageDown,
            change_camera: KeyCode::KeyC,
        }
    }
}

pub fn input_system(
    mut input: ResMut<InputAxis>,
    keys: Res<ButtonInput<KeyCode>>,
    keymap: Res<Keymap>,
    settings: Res<Settings>,
    keyboard_input: Res<'_, ButtonInput<KeyCode>>,
    gp: Option<Single<&Gamepad>>,
    mut camera_settings: ResMut<CameraSettings>,
) {
    let mut gamepad_input = InputAxis {
        pitch: 0.,
        roll: 0.,
        yaw: 0.,
        throttle: 0.,
    };

    if keyboard_input.just_pressed(keymap.change_camera) {
        if camera_settings.view == 0 {
            camera_settings.view = 1
        } else if camera_settings.view == 1 {
            camera_settings.view = 0
        }
    }

    if settings.gamepad.enabled {
        let gamepad = gp.expect("gamepad.enabled set to true but no gamepad detected.");
        if let (Some(x), Some(y)) = (
            gamepad.get(GamepadAxis::LeftStickX),
            gamepad.get(GamepadAxis::LeftStickY),
        ) {
            gamepad_input.pitch = -y;
            gamepad_input.roll = -x;
        }

        if gamepad.just_pressed(GamepadButton::DPadDown) {
            gamepad_input.throttle = -0.1;
        }
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            gamepad_input.throttle = 0.1;
        }

        if gamepad.pressed(GamepadButton::DPadLeft) {
            gamepad_input.yaw = 1.0;
        }
        if gamepad.pressed(GamepadButton::DPadRight) {
            gamepad_input.yaw = -1.0;
        }

        input.pitch = gamepad_input.pitch;
        input.roll = gamepad_input.roll;
        input.yaw = gamepad_input.yaw;
        input.throttle += gamepad_input.throttle;
        input.throttle = input.throttle.clamp(0., 1.);
    } else {
        let mut button_input = InputAxis {
            pitch: 0.,
            roll: 0.,
            yaw: 0.,
            throttle: 0.,
        };
        if keys.pressed(keymap.up) {
            button_input.pitch = -1.0
        }
        if keys.pressed(keymap.down) {
            button_input.pitch = 1.0
        }
        if keys.pressed(keymap.roll_left) {
            button_input.roll = 1.0
        }
        if keys.pressed(keymap.roll_right) {
            button_input.roll = -1.0
        }

        if keys.pressed(keymap.rudder_left) {
            button_input.yaw = 1.0
        }
        if keys.pressed(keymap.rudder_right) {
            button_input.yaw = -1.0
        }

        if keys.just_pressed(keymap.throttle_up) {
            button_input.throttle = 0.1
        }
        if keys.just_pressed(keymap.throttle_down) {
            button_input.throttle = -0.1
        }

        input.pitch = button_input.pitch;
        input.roll = button_input.roll;
        input.yaw = button_input.yaw;
        input.throttle += button_input.throttle;
        input.throttle = input.throttle.clamp(0.0, 1.0);
    }
}
