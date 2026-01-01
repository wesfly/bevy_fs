use crate::{InputAxis, Settings};
use bevy::{
    input::{gamepad::GamepadEvent, keyboard::KeyboardInput},
    prelude::*,
};

#[derive(Resource)]
pub struct GamepadSettings {
    control_snapping_enabled: bool,
    control_snapping_range: std::ops::Range<f32>,
}

impl Default for GamepadSettings {
    fn default() -> Self {
        Self {
            control_snapping_enabled: true,
            control_snapping_range: -0.075..0.075,
        }
    }
}

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
        }
    }
}
// pitch roll yaw throttle
pub fn input_system(
    mut gamepad_events: MessageReader<GamepadEvent>,
    gamepad_settings: Res<GamepadSettings>,
    mut input: ResMut<InputAxis>,
    mut keyboard_events: MessageReader<KeyboardInput>,
    keymap: Res<Keymap>,
    settings: Res<Settings>,
) {
    let mut gamepad_input = InputAxis {
        pitch: 0.,
        roll: 0.,
        yaw: 0.,
        throttle: 0.,
    };

    if settings.gamepad_enabled {
        for event in gamepad_events.read() {
            match event {
                GamepadEvent::Connection(e) => info!("Gamepad connection: {:?}", e),
                GamepadEvent::Button(e) => {
                    if e.button == GamepadButton::DPadLeft {
                        gamepad_input.yaw = 1.;
                    } else if e.button == GamepadButton::DPadRight {
                        gamepad_input.yaw = -1.
                    } else {
                        gamepad_input.yaw = 0.
                    }

                    if e.button == GamepadButton::DPadUp {
                        gamepad_input.throttle = 0.1;
                    } else if e.button == GamepadButton::DPadDown {
                        gamepad_input.throttle = -0.1
                    }
                }
                GamepadEvent::Axis(e) => {
                    if e.axis == GamepadAxis::LeftStickX {
                        gamepad_input.roll = clamp_input_value(-e.value, &gamepad_settings)
                    }
                    if e.axis == GamepadAxis::LeftStickY {
                        gamepad_input.pitch = clamp_input_value(-e.value, &gamepad_settings)
                    }
                }
            }
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

        for event in keyboard_events.read() {
            match event.key_code {
                a if a == keymap.up => button_input.pitch = -1.,
                a if a == keymap.down => button_input.pitch = 1.,
                a if a == keymap.roll_left => button_input.roll = 1.,
                a if a == keymap.roll_right => button_input.roll = -1.,
                a if a == keymap.rudder_right => button_input.yaw = -1.,
                a if a == keymap.rudder_left => button_input.yaw = 1.,
                a if a == keymap.throttle_up => button_input.throttle = 0.1,
                a if a == keymap.throttle_down => button_input.throttle = -0.1,

                _ => {}
            }
        }

        input.pitch = button_input.pitch;
        input.roll = button_input.roll;
        input.yaw = button_input.yaw;
        input.throttle += button_input.throttle;
    }
}

fn clamp_input_value(value: f32, gamepad_settings: &Res<GamepadSettings>) -> f32 {
    if gamepad_settings.control_snapping_enabled {
        if gamepad_settings.control_snapping_range.contains(&value) {
            return 0.;
        }
    }
    value
}
