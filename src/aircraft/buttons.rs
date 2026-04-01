use crate::aircraft::landing_gear::LandingGearStatus;

use super::AircraftState;
use bevy::prelude::*;
use serde::Deserialize;

#[derive(Component, Debug, Deserialize, PartialEq)]
pub enum InterfaceOperation {
    AntiColLt,
    Engine,
    PositionLt,
    StrobeLt,
    FormationLt,
    Apu,
    LdgGear,
}

#[derive(Debug, Component, Deserialize, PartialEq)]
pub enum InterfaceType {
    Switch,
    Button,
    Lever,
}

#[derive(Debug, Component, Deserialize, PartialEq)]
pub struct Button {
    pub interface_type: InterfaceType,
    pub operation: Option<InterfaceOperation>,
    pub inverse: Option<bool>,
}

#[derive(Message)]
pub struct ButtonMessages(pub InterfaceOperation);

impl Button {
    pub fn listener(
        mut query_tf_button: Query<(&mut Transform, &Button)>,
        mut state: ResMut<AircraftState>,
        mut messages: MessageReader<ButtonMessages>,
        ldg_gear_status: Res<LandingGearStatus>,
    ) {
        for message in messages.read() {
            let interface_op = &message.0;
            let (bool, _) = match interface_op {
                InterfaceOperation::Engine => {
                    (Some(state.engine_on), state.engine_on = !state.engine_on)
                }
                InterfaceOperation::AntiColLt => (
                    Some(state.anti_col_lts_on),
                    state.anti_col_lts_on = !state.anti_col_lts_on,
                ),
                InterfaceOperation::PositionLt => {
                    (Some(state.pos_lts_on), state.pos_lts_on = !state.pos_lts_on)
                }
                InterfaceOperation::StrobeLt => (
                    Some(state.strobe_lts_on),
                    state.strobe_lts_on = !state.strobe_lts_on,
                ),
                InterfaceOperation::FormationLt => (
                    Some(state.form_lts_on),
                    state.form_lts_on = !state.form_lts_on,
                ),
                InterfaceOperation::LdgGear => (
                    match *ldg_gear_status {
                        LandingGearStatus::Deploying => None,
                        LandingGearStatus::Deployed => Some(state.landing_gear_deployed),
                        LandingGearStatus::Retracting => None,
                        LandingGearStatus::Retracted => Some(state.landing_gear_deployed),
                    },
                    (),
                ),
                _ => (None, ()),
            };
            for (mut transform, button) in &mut query_tf_button {
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

                    if *interface_op == *button.operation.as_ref().unwrap() {
                        transform.rotate_local_x(angle.to_radians());
                    }
                }
            }
        }
    }
}
