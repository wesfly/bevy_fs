use crate::{
    Aircraft, InputAxis,
    data_from_gltf::{ButtonID, ButtonTypes, Lights},
};
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Resource)]
pub struct AircraftState {
    pub engine_on: bool,
    pub anti_col_lts_on: bool,
    pub pos_lts_on: bool,
}

impl Default for AircraftState {
    fn default() -> Self {
        Self {
            engine_on: false,
            anti_col_lts_on: false,
            pos_lts_on: false,
        }
    }
}

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&crate::data_from_gltf::Button>,
    mut transform: Query<&mut Transform, With<crate::data_from_gltf::Button>>,
    mut state: ResMut<AircraftState>,
) {
    let button = function_comps.get(press.entity.entity()).unwrap();
    let bool;
    match button.function.as_ref().unwrap() {
        ButtonID::Engine => {
            bool = Some(state.engine_on);
            state.engine_on = !state.engine_on
        }
        ButtonID::AntiColLt => {
            bool = Some(state.anti_col_lts_on);
            state.anti_col_lts_on = !state.anti_col_lts_on
        }
        ButtonID::PositionLt => {
            bool = Some(state.pos_lts_on);
            state.pos_lts_on = !state.pos_lts_on
        }
        _ => {
            warn!("This button isn't implemented yet. Do it yourself or wait. =)");
            bool = None;
        }
    }

    const SWITCH_ANGLE_LIMIT: f32 = 70.0;
    match button.button {
        ButtonTypes::Switch => {
            if let Some(mut bool) = bool {
                if let Some(inverse) = button.inverse {
                    if inverse {
                        bool = !bool
                    }
                }

                let angle: f32;
                match bool {
                    true => angle = -SWITCH_ANGLE_LIMIT,
                    false => angle = SWITCH_ANGLE_LIMIT,
                }
                transform
                    .get_mut(press.entity.entity())
                    .unwrap()
                    .rotate_local_x(angle.to_radians());
            }
        }
        _ => {}
    }
}

pub fn update_lights(
    material_handles: Query<(&MeshMaterial3d<StandardMaterial>, &Lights), With<Lights>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<AircraftState>,
) {
    #[allow(irrefutable_let_patterns)] // Acting like I know what I'm doing
    for material_handle in material_handles.iter() {
        if let Some(material) = materials.get_mut(material_handle.0)
            && let LinearRgba {
                ref mut red,
                ref mut green,
                ref mut blue,
                alpha: _,
            } = material.emissive
        {
            match material_handle.1 {
                Lights::AntiCol => {
                    if *red == 0.0 && state.anti_col_lts_on {
                        *red = 100.
                    } else {
                        *red = 0.0
                    }
                }
                Lights::PositionPort => {
                    if state.pos_lts_on {
                        *green = 100.
                    } else {
                        *green = 0.0
                    }
                }
                Lights::PositionStarboard => {
                    if state.pos_lts_on {
                        *red = 100.
                    } else {
                        *red = 0.0
                    }
                }
                Lights::PositionRear => {
                    if state.pos_lts_on {
                        *red = 100.;
                        *green = 100.;
                        *blue = 100.
                    } else {
                        *red = 0.;
                        *green = 0.;
                        *blue = 0.
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut query: Query<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
    state: Res<AircraftState>,
) {
    if state.engine_on {
        let thrust_factor = 64_000.;
        let force = transform.up() * thrust_factor * (input.throttle);
        let torque = Vec3::new(input.pitch, input.yaw, input.roll);

        for mut forces in &mut query {
            forces.apply_force(force);
            forces.apply_local_torque(torque * 500.0);
        }
    }
}
