use crate::{
    Settings,
    data_from_gltf::{InterfaceOperation, InterfaceType, Lights, RotorTypes, load},
    input::InputAxis,
    motion_blur,
};
use avian3d::prelude::*;
use bevy::{
    camera::Exposure,
    core_pipeline::tonemapping::Tonemapping,
    light::AtmosphereEnvironmentMapLight,
    pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    render::view::Hdr,
};
use serde::Deserialize;
use std::time::Duration;

pub const STROBE_OFF_DURATION: f32 = 1.0;
pub const STROBE_ON_DURATION: f32 = 0.1;
pub const ACOL_OFF_DURATION: f32 = 1.2;
pub const ACOL_ON_DURATION: f32 = 0.1;

#[derive(Resource, Default, Deserialize)]
pub enum AircraftTypes {
    Helicopter,
    #[default]
    Aeroplane,
}

#[derive(Resource)]
pub struct AircraftState {
    pub engine_on: bool,
    pub anti_col_lts_on: bool,
    pub pos_lts_on: bool,
    pub strobe_lts_on: bool,
    pub aircraft_type: AircraftTypes,
}

#[derive(Component)]
pub struct Aircraft;

pub fn button_listener(
    press: On<Pointer<Press>>,
    function_comps: Query<&crate::data_from_gltf::Button>,
    mut transform: Query<&mut Transform, With<crate::data_from_gltf::Button>>,
    mut state: ResMut<AircraftState>,
) {
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

#[derive(Resource)]
pub struct LightsTimers {
    pub acol: Timer,
    pub acol_on_cycle: bool,
    pub strobe: Timer,
    pub strobe_on_cycle: bool,
}

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
            }
        }
    }
}

pub fn update_lights(
    state: Res<AircraftState>,
    timer: ResMut<LightsTimers>,
    query: Query<(&mut PointLight, &Lights)>,
) {
    for (mut point_light, light) in query {
        let (colour, on) = match light {
            Lights::PositionPort => (Color::linear_rgb(1.0, 0.0, 0.0), state.pos_lts_on),
            Lights::PositionStarboard => (Color::linear_rgb(0.0, 1.0, 0.0), state.pos_lts_on),
            Lights::PositionRear => (Color::linear_rgb(1.0, 1.0, 1.0), state.pos_lts_on),

            Lights::AntiCol => (
                Color::linear_rgb(1.0, 0.0, 0.0),
                (state.anti_col_lts_on && timer.acol_on_cycle),
            ),

            Lights::Strobe => (
                Color::linear_rgb(1.0, 1.0, 1.0),
                (state.strobe_lts_on && timer.strobe_on_cycle),
            ),
        };

        point_light.color = colour;

        if on {
            point_light.intensity = 10000.0;
        } else {
            point_light.intensity = 0.0
        }
    }
}

pub fn update_rotors(
    query: Query<(&mut Transform, &RotorTypes)>,
    state: Res<AircraftState>,
    time: Res<Time>,
) {
    if state.engine_on {
        for (mut rotor, rotor_type) in query {
            match rotor_type {
                RotorTypes::Main => rotor.rotate_local_y(100.0 * time.delta_secs()),
                RotorTypes::Rear => rotor.rotate_local_z(100.0 * time.delta_secs()),
            }
        }
    }
}

const ASPECT_RATIO: f32 = 1.0;

pub fn mechanics(
    transform: Single<&GlobalTransform, With<Aircraft>>,
    mut force: Single<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
    state: Res<AircraftState>,
) {
    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            if state.engine_on {
                let thrust_factor = 64_000.;
                let thrust = transform.up() * thrust_factor * input.throttle;
                let torque = Vec3::new(input.pitch, input.yaw, input.roll);

                force.apply_force(thrust);
                force.apply_local_torque(torque * 500.0);
            }
        }
        AircraftTypes::Aeroplane => {
            let input_torque = Vec3::new(input.pitch, input.yaw, input.roll);
            force.apply_local_torque(input_torque * 500.5);

            let forward = transform.forward();

            let rho = rho();

            let velocity = force.linear_velocity();
            let velocity_dir = velocity.normalize_or_zero();
            let speed: f32 = velocity.length();

            force.apply_force(thrust(&input, &forward));

            // Angle of attack
            let sin = forward.cross(velocity_dir).dot(transform.right().as_vec3());
            let cos = forward.dot(velocity_dir);
            let aoa = -sin.atan2(cos).to_degrees();

            let lift_coeff = match aoa {
                d if d < 15.0 => d / 15.0 * 1.0 + 0.5,
                d if d < 20.0 => 1.2 * (1.0 - (d - 15.0) / 5.0),
                _ => 0.2, // stalled
            };

            let parasitic_drag = velocity.powf(2.0) * 0.8 * forward.cross(velocity_dir);
            let drag = (-velocity_dir * induced_drag(lift_coeff, rho, speed))
                + (-velocity_dir * parasitic_drag);
            force.apply_force(drag);
            info!("drag {:#?}", drag);

            // Stabilisation (idk)
            let stability_thing = stabilise(&transform, velocity_dir, speed);
            force.apply_local_angular_acceleration(stability_thing);

            // L = Cl * p * (v^2/2) * A
            // Lift = coefficient * density * (airspeed^2 / 2) * wing area
            let wing_area = 49.0;
            let airspeed = forward.dot(velocity_dir).clamp(0.0, 1.0) * speed;
            let lift = lift(lift_coeff, airspeed, wing_area, transform.up(), rho);
            info!("lift {:#?}", lift);
            force.apply_force(lift);
        } // }
    }
}

fn rho() -> f32 {
    // TODO implement that whole air pressure stuff
    1.2041
}

fn thrust(input: &InputAxis, forward: &Dir3) -> Vec3 {
    let thrust_factor = 150_000.0;
    let thrust = forward.as_vec3() * thrust_factor * input.throttle;

    thrust
}

fn induced_drag(lift_coeff: f32, rho: f32, speed: f32) -> f32 {
    let zero_lift_induced_drag_coeff = 0.0;
    let induced_drag_coeff =
        zero_lift_induced_drag_coeff + lift_coeff.powi(2) / std::f32::consts::PI * ASPECT_RATIO;
    let wingspan: f32 = 15.0;
    let induced_drag = 0.5 * rho * speed.powi(2) * induced_drag_coeff * wingspan.powi(2);

    induced_drag
}

fn stabilise(transform: &GlobalTransform, velocity_dir: Vec3, speed: f32) -> Vec3 {
    let stability_error = transform.forward().cross(velocity_dir);
    let local_error = transform.rotation().inverse().mul_vec3(stability_error);
    let snap_intensity = 0.0015;
    let mut stability_torque = local_error * speed.powi(2) * snap_intensity;
    stability_torque.x = 0.0;

    stability_torque
}

fn lift(lift_coeff: f32, airspeed: f32, wing_area: f32, up: Dir3, rho: f32) -> Vec3 {
    let lift_force = lift_coeff * rho * (airspeed.powi(2) * 0.5) * wing_area;
    let lift_vector = lift_force * up;
    lift_vector
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    settings: &Res<Settings>,
    state: Res<AircraftState>,
) {
    let path = match state.aircraft_type {
        AircraftTypes::Aeroplane => "rafael.glb",
        AircraftTypes::Helicopter => "aircraft.glb",
    };

    // Aircraft collider
    let aircraft = commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(1).from_asset(path))),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            Transform::from_xyz(0., 2000., 0.),
            Mass(10_000.0),
            LinearVelocity(Vec3 {
                x: 0.0,
                y: 0.0,
                z: -100.0,
            }),
            Visibility::Hidden,
        ))
        .id();

    // The real aircraft model
    commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Visibility::Visible,
            ChildOf(aircraft),
            ColliderDisabled,
            RigidBodyDisabled,
        ))
        .observe(load);

    let mut camera = commands.spawn((
        Camera3d::default(),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereEnvironmentMapLight::default(),
        AtmosphereSettings::default(),
        Exposure::SUNLIGHT,
        Tonemapping::AgX,
        Bloom::NATURAL,
        Projection::from(PerspectiveProjection {
            fov: 50.0_f32.to_radians(),
            ..default()
        }),
        Hdr,
        crate::camera::Camera,
        ChildOf(aircraft),
    ));

    // TODO make this a plugin
    if let Some(sse) = crate::sse_config(settings) {
        camera.insert(sse);
    }

    if let Some(mb) = motion_blur(settings) {
        camera.insert(mb);
    }
}
