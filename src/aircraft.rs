use crate::data_from_gltf::ButtonID;
use bevy::prelude::*;
#[derive(Resource)]
pub struct AircraftState {
    pub engine_on: bool,
    pub anti_col_lights_on: bool,
}

impl Default for AircraftState {
    fn default() -> Self {
        Self {
            engine_on: false,
            anti_col_lights_on: false,
        }
    }
}

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&ButtonID>,
    mut state: ResMut<AircraftState>,
) {
    // TODO add button animation
    let button_id = function_comps.get(press.entity.entity()).unwrap();
    match button_id {
        ButtonID::Engine => state.engine_on = !state.engine_on,
        ButtonID::AntiCol => state.anti_col_lights_on = !state.anti_col_lights_on,
        _ => {
            info!("This button isn't implemented yet. Do it yourself or wait. =)")
        }
    }
}

pub fn update_anti_col(
    material_handles: Query<&MeshMaterial3d<StandardMaterial>, With<crate::data_from_gltf::Lights>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<AircraftState>,
) {
    #[allow(irrefutable_let_patterns)] // Acting like I know what I'm doing
    for material_handle in material_handles.iter() {
        if let Some(material) = materials.get_mut(material_handle)
            && let LinearRgba {
                ref mut red,
                green: _,
                blue: _,
                alpha: _,
            } = material.emissive
        {
            if *red == 0.0 && state.anti_col_lights_on {
                *red = 100.
            } else {
                *red = 0.0
            }
        }
    }
}
