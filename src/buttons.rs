use crate::{AircraftState, handle_custom_properties::ButtonID};
use bevy::prelude::*;

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&ButtonID>,
    mut state: ResMut<AircraftState>,
) {
    // TODO add button animation
    let button_id = function_comps.get(press.entity.entity()).unwrap();
    match button_id {
        ButtonID::Button1 => state.engine_on = !state.engine_on,
        ButtonID::Button2 => {
            // TODO implement lights
            info!("Lights aren't implemented yet.")
        }

        _ => {
            info!("This button isn't implemented yet. Do it yourself or wait.")
        }
    }
}
