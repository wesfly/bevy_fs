use super::AircraftState;
use crate::aircraft::breeze::landing_gear::{LandingGearCommand, LandingGearStatus};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy::{
    picking::{hover::HoverMap, pointer::PointerId},
    window::CursorIcon,
};
use serde::Deserialize;

#[derive(Component, Clone, Debug, Deserialize, PartialEq)]
pub enum InterfaceOperation {
    AntiColLt,
    Engine,
    PositionLt,
    StrobeLt,
    FormationLt,
    Apu,
    LdgGear,
    ParkBrk,
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

pub fn update_cursor(
    hover_map: Res<HoverMap>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<Entity, With<PrimaryWindow>>,
    mut commands: Commands,
    grabbable_query: Query<&Button>,
) {
    let Ok(window_entity) = window_query.single() else {
        return;
    };

    let hovered_target = hover_map
        .get(&PointerId::Mouse)
        .and_then(|hovered_entities| hovered_entities.iter().next())
        .map(|(entity, _depth)| *entity);

    let is_hovering_target = hovered_target
        .map(|entity| grabbable_query.contains(entity))
        .unwrap_or(false);

    if is_hovering_target {
        if mouse_input.pressed(MouseButton::Left) {
            commands
                .entity(window_entity)
                .insert(CursorIcon::System(bevy::window::SystemCursorIcon::Grab));
        } else {
            commands
                .entity(window_entity)
                .insert(CursorIcon::System(bevy::window::SystemCursorIcon::Pointer));
        }
    } else {
        commands
            .entity(window_entity)
            .insert(CursorIcon::System(bevy::window::SystemCursorIcon::Default));
    }
}

impl Button {
    pub fn press_observer(
        press: On<Pointer<Press>>,
        function_comps: Query<&Button>,
        mut messages: MessageWriter<ButtonMessages>,
    ) {
        let button = function_comps.get(press.entity.entity()).unwrap();
        messages.write(ButtonMessages(button.operation.clone().unwrap()));
    }

    pub fn listener(
        mut query_tf_button: Query<(&mut Transform, &Button)>,
        mut state: ResMut<AircraftState>,
        mut messages: MessageReader<ButtonMessages>,
        ldg_gear_status: Res<LandingGearStatus>,
        mut landing_gear_messages: MessageWriter<LandingGearCommand>,
    ) {
        for message in messages.read() {
            let interface_op = &message.0;
            let (bool, _) = match interface_op {
                InterfaceOperation::Engine => {
                    (Some(state.engine.on), state.engine.on = !state.engine.on)
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
                        LandingGearStatus::Deploying => Some(true),
                        LandingGearStatus::Deployed => Some(state.landing_gear_deployed),
                        LandingGearStatus::Retracting => Some(false),
                        LandingGearStatus::Retracted => Some(state.landing_gear_deployed),
                    },
                    {
                        landing_gear_messages.write(LandingGearCommand(
                            super::breeze::landing_gear::LandingGearCommands::Toggle,
                        ));
                    },
                ),
                InterfaceOperation::ParkBrk => (
                    Some(state.parking_brake),
                    state.parking_brake = !state.parking_brake,
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
