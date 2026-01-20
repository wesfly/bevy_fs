use crate::{AircraftState, handle_custom_properties::ButtonID};
use bevy::prelude::*;

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&ButtonID>,
    mut state: ResMut<AircraftState>,
) {
    let button_type = function_comps.get(press.entity.entity());
    match button_type.unwrap() {
        ButtonID::Button1 => state.engine_on = false,
        ButtonID::Button2 => state.engine_on = true,

        _ => {}
    }
}
