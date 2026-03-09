use super::AircraftState;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Debug, Deserialize)]
pub enum InterfaceOperation {
    AntiColLt,
    Engine,
    PositionLt,
    StrobeLt,
    FormationLt,
    Apu,
}

#[derive(Debug, Component, Deserialize)]
pub enum InterfaceType {
    Switch,
    Button,
    Lever,
}

#[derive(Debug, Component, Deserialize)]
pub struct Button {
    pub interface_type: InterfaceType,
    pub operation: Option<InterfaceOperation>,
    pub inverse: Option<bool>,
}

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&Button>,
    mut transform: Query<&mut Transform, With<Button>>,
    mut state: ResMut<AircraftState>,
) {
    if press.button == PointerButton::Primary {
        let button = function_comps.get(press.entity.entity()).unwrap();
        let bool;
        match button.operation.as_ref().unwrap() {
            InterfaceOperation::Engine => {
                bool = Some(state.engine_on);
                state.engine_on = !state.engine_on
            }
            InterfaceOperation::AntiColLt => {
                bool = Some(state.anti_col_lts_on);
                state.anti_col_lts_on = !state.anti_col_lts_on
            }
            InterfaceOperation::PositionLt => {
                bool = Some(state.pos_lts_on);
                state.pos_lts_on = !state.pos_lts_on
            }
            InterfaceOperation::StrobeLt => {
                bool = Some(state.strobe_lts_on);
                state.strobe_lts_on = !state.strobe_lts_on
            }
            _ => bool = None,
        }

        const SWITCH_ANGLE_LIMIT: f32 = 70.0;
        if let InterfaceType::Switch = button.interface_type
            && let Some(mut bool) = bool
        {
            if let Some(inverse) = button.inverse
                && inverse
            {
                bool = !bool
            }

            let angle = match bool {
                true => -SWITCH_ANGLE_LIMIT,
                false => SWITCH_ANGLE_LIMIT,
            };
            transform
                .get_mut(press.entity.entity())
                .unwrap()
                .rotate_local_x(angle.to_radians());
        }
    }
}
