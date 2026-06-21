use crate::aircraft::AircraftState;
use bevy::prelude::*;
use serde::Deserialize;
use std::time::Duration;

pub const STROBE_OFF_DURATION: f32 = 1.0;
pub const STROBE_ON_DURATION: f32 = 0.1;
pub const ACOL_OFF_DURATION: f32 = 1.2;
pub const ACOL_ON_DURATION: f32 = 0.1;

#[derive(Deserialize, Debug, Component)]
pub enum Lights {
    AntiCol,
    Strobe,
    PositionPort,
    PositionStarboard,
    PositionRear,
    Formation,
    Landing,
}

#[derive(Debug, Deserialize)]
pub struct Light {
    pub light: Lights,
}

#[derive(Resource)]
pub struct LightsTimers {
    pub acol: Timer,
    pub acol_on_cycle: bool,
    pub strobe: Timer,
    pub strobe_on_cycle: bool,
}

impl Light {
    pub fn update_light_cycle(time: Res<Time>, mut timer: ResMut<LightsTimers>) {
        let delta = time.delta();
        if timer.acol.just_finished() && !timer.acol_on_cycle {
            timer.acol_on_cycle = true;
            timer
                .acol
                .set_duration(Duration::from_secs_f32(ACOL_ON_DURATION));
        } else if timer.acol.just_finished() && timer.acol_on_cycle {
            timer.acol_on_cycle = false;
            timer
                .acol
                .set_duration(Duration::from_secs_f32(ACOL_OFF_DURATION));
        }

        if timer.strobe.just_finished() && !timer.strobe_on_cycle {
            timer.strobe_on_cycle = true;
            timer
                .strobe
                .set_duration(Duration::from_secs_f32(STROBE_ON_DURATION));
        } else if timer.strobe.just_finished() && timer.strobe_on_cycle {
            timer.strobe_on_cycle = false;
            timer
                .strobe
                .set_duration(Duration::from_secs_f32(STROBE_OFF_DURATION));
        }

        timer.acol.tick(delta);
        timer.strobe.tick(delta);
    }

    pub fn update_mesh_lights(
        material_handles: Query<(&MeshMaterial3d<StandardMaterial>, &Lights, Entity), With<Lights>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        state: Res<AircraftState>,
        timer: ResMut<LightsTimers>,
    ) {
        for material_handle in material_handles.iter() {
            if let Some(mut material) = materials.get_mut(material_handle.0) {
                let LinearRgba {
                    ref mut red,
                    ref mut green,
                    ref mut blue,
                    alpha: _,
                } = material.emissive;

                match material_handle.1 {
                    Lights::AntiCol => {
                        if state.anti_col_lts_on && timer.acol_on_cycle {
                            *red = 100.0
                        } else {
                            *red = 0.0
                        }
                    }
                    Lights::PositionPort => {
                        if state.pos_lts_on {
                            *red = 100.
                        } else {
                            *red = 0.0
                        }
                    }
                    Lights::PositionStarboard => {
                        if state.pos_lts_on {
                            *green = 100.
                        } else {
                            *green = 0.0
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
                    Lights::Strobe => {
                        if state.strobe_lts_on && timer.strobe_on_cycle {
                            *red = 100.;
                            *green = 100.;
                            *blue = 100.
                        } else {
                            *red = 0.;
                            *green = 0.;
                            *blue = 0.
                        }
                    }
                    Lights::Formation => {
                        if state.form_lts_on {
                            *red = 10.;
                            *green = 20.;
                            *blue = 0.
                        } else {
                            *red = 0.;
                            *green = 0.;
                            *blue = 0.
                        }
                    }
                    Lights::Landing => {
                        if state.ldg_lts_on && state.landing_gear_deployed {
                            *red = 500.;
                            *green = 500.;
                            *blue = 500.
                        } else {
                            *red = 0.;
                            *green = 0.;
                            *blue = 0.
                        }
                    }
                }
            }
        }
    }

    pub fn update_lights(
        state: Res<AircraftState>,
        timer: ResMut<LightsTimers>,
        point_light_query: Query<(&mut PointLight, &Lights)>,
        spot_light_query: Query<(&mut SpotLight, &Lights)>,
    ) {
        for (mut point_light, light) in point_light_query {
            let (colour, on) = match light {
                Lights::PositionPort => (Color::linear_rgb(10.0, 0.0, 0.0), state.pos_lts_on),
                Lights::PositionStarboard => (Color::linear_rgb(0.0, 10.0, 0.0), state.pos_lts_on),
                Lights::PositionRear => (Color::linear_rgb(1.0, 1.0, 1.0), state.pos_lts_on),
                Lights::AntiCol => (
                    Color::linear_rgb(1.0, 0.0, 0.0),
                    (state.anti_col_lts_on && timer.acol_on_cycle),
                ),
                Lights::Strobe => (
                    Color::linear_rgb(1.0, 1.0, 1.0),
                    (state.strobe_lts_on && timer.strobe_on_cycle),
                ),

                _ => (Color::linear_rgb(1.0, 1.0, 1.0), false),
            };

            point_light.color = colour;

            if on {
                point_light.intensity = 10000.0;
            } else {
                point_light.intensity = 0.0
            }
        }
        for (mut spot_light, light) in spot_light_query {
            let (colour, on) = match light {
                Lights::Landing => (
                    Color::linear_rgb(1.0, 1.0, 1.0),
                    state.ldg_lts_on && state.landing_gear_deployed,
                ),
                _ => (Color::BLACK, false),
            };

            spot_light.color = colour;

            if on {
                spot_light.intensity = 100000000.0;
            } else {
                spot_light.intensity = 0.0
            }
        }
    }
}
