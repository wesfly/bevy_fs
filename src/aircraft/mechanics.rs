// Thanks to Hermitao for making a prototype flight model (https://gist.github.com/Hermitao/0a908f8af19b11132e3bdb5ba4ef99f0)

use crate::{
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    input::InputAxis,
};
use avian3d::prelude::{
    Forces, LinearVelocity, ReadRigidBodyForces, WriteRigidBodyForces, forces::ForcesItem,
};
use bevy::prelude::*;

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
                let thrust_factor = 120_000.;
                let thrust = transform.up() * thrust_factor * input.throttle;
                let torque = Vec3::new(input.pitch, input.yaw, input.roll);

                force.apply_force(thrust);
                force.apply_local_torque(torque * 500.0);
            }
        }
        AircraftTypes::Aeroplane => {
            if state.engine_on {
                steering(*transform, &mut force, &input);

                let forward = transform.forward();

                let rho = rho(transform.translation().y as f64).density as f32;

                let velocity = force.linear_velocity();
                let velocity_dir = velocity.normalize_or_zero();
                let speed: f32 = velocity.length();

                force.apply_force(thrust(&input, &forward) * rho / 1.2041);

                // Angle of attack
                let sin = forward.cross(velocity_dir).dot(transform.right().as_vec3());
                let cos = forward.dot(velocity_dir);
                let alpha = -sin.atan2(cos).to_degrees();

                let lift_coeff = lift_coeff(alpha);

                let parasitic_drag_coeff = match state.landing_gear_deployed {
                    true => 2.2,
                    false => 1.8,
                };
                let parasitic_drag =
                    parasitic_drag_coeff * velocity.powf(2.0) + 0.5 * forward.cross(velocity_dir);

                let drag = (-velocity_dir * induced_drag(lift_coeff, rho, speed))
                    + (-velocity_dir * parasitic_drag);
                // dbg!(drag);
                force.apply_force(drag);

                // Stabilisation (idk)
                stabilise();

                // L = Cl * p * (v^2/2) * A
                // Lift = coefficient * density * (airspeed^2 / 2) * wing area
                let wing_area = 49.0;
                let airspeed = forward.dot(velocity_dir).clamp(0.0, 1.0) * speed;
                let lift = lift(lift_coeff, airspeed, wing_area, transform.up(), rho);
                force.apply_force(lift);
            }
        }
    }
}

struct AircraftPhysicsConfig {
    pitch_point: Vec3,
    yaw_point: Vec3,
    roll_port_point: Vec3,
    roll_starboard_point: Vec3,
}

fn steering(transform: &GlobalTransform, force: &mut ForcesItem, input: &InputAxis) {
    let airspeed = transform.forward().dot(force.linear_velocity());
    let factor = 0.01;

    let physics_cfg = AircraftPhysicsConfig {
        pitch_point: Vec3 {
            x: 0.0,
            y: 0.0,
            z: 10.0,
        },
        yaw_point: Vec3 {
            x: 0.0,
            y: 2.0,
            z: 7.0,
        },
        roll_port_point: Vec3 {
            x: -6.0,
            y: 0.0,
            z: 2.0,
        },
        roll_starboard_point: Vec3 {
            x: 6.0,
            y: 0.0,
            z: 2.0,
        },
    };

    let pitch_point = transform.translation() + transform.rotation() * physics_cfg.pitch_point;
    let yaw_point = transform.translation() + transform.rotation() * physics_cfg.yaw_point;

    let roll_port_point =
        transform.translation() + transform.rotation() * physics_cfg.roll_port_point;
    let roll_port_force = Vec3 {
        x: 0.0,
        y: -input.roll * 50.,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * roll_port_force * airspeed * factor,
        roll_port_point,
    );

    let roll_starboard_point =
        transform.translation() + transform.rotation() * physics_cfg.roll_starboard_point;
    let roll_starboard_force = Vec3 {
        x: 0.0,
        y: input.roll * 50.,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * roll_starboard_force * airspeed * factor,
        roll_starboard_point,
    );

    let pitch_force = Vec3 {
        x: 0.0,
        y: -input.pitch * 50.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * pitch_force * airspeed * factor,
        pitch_point,
    );

    let yaw_force = Vec3 {
        x: input.yaw * 50.0,
        y: 0.0,
        z: 0.0,
    };
    force.apply_force_at_point(
        transform.rotation() * yaw_force * airspeed * factor,
        yaw_point,
    );
}

fn lift_coeff(alpha_deg: f32) -> f32 {
    let alpha = alpha_deg.to_radians();

    // Polhamus Analogy Constants
    let potential_lift_factor = 1.65;
    let vortex_lift_factor = 3.05;

    let sin_a = alpha.sin();
    let cos_a = alpha.cos();

    // Polhamus Equation
    let cl_potential = potential_lift_factor * sin_a * cos_a.powi(2);
    let cl_vortex = vortex_lift_factor * sin_a.powi(2) * cos_a;

    cl_potential + cl_vortex
}

#[allow(unused)]
#[derive(Debug)]
pub struct Atmosphere {
    pub pressure: f64,    // Pascals (Pa)
    pub density: f64,     // kg/m^3 (Rho)
    pub temperature: f64, // Kelvin (K)
}

pub fn rho(altitude_m: f64) -> Atmosphere {
    // Constants defined by the US Standard Atmosphere 1976
    const G0: f64 = 9.80665; // Gravity at sea level [m/s^2]
    const M: f64 = 0.0289644; // Molar mass of Earth's air [kg/mol]
    const R: f64 = 8.31432; // Universal gas constant [J/(mol*K)]
    const EARTH_RADIUS: f64 = 6356766.0; // Earth radius [m]

    // Atmospheric layers: (Base Geopotential Altitude, Base Pressure, Base Temp, Lapse Rate)
    // Data covers up to 71 km (71,000 m)
    const LAYERS: [(f64, f64, f64, f64); 6] = [
        (0.0, 101325.0, 288.15, -0.0065),    // Troposphere
        (11000.0, 22632.1, 216.65, 0.0),     // Tropopause
        (20000.0, 5474.89, 216.65, 0.001),   // Lower Stratosphere
        (32000.0, 868.019, 228.65, 0.0028),  // Upper Stratosphere
        (47000.0, 110.906, 270.65, 0.0),     // Stratopause
        (51000.0, 66.9388, 270.65, -0.0028), // Lower Mesosphere
    ];

    // Convert geometric altitude to geopotential altitude
    let h = (EARTH_RADIUS * altitude_m) / (EARTH_RADIUS + altitude_m);

    // Find the correct atmospheric layer
    let mut base_h = LAYERS[0].0;
    let mut base_p = LAYERS[0].1;
    let mut base_t = LAYERS[0].2;
    let mut lapse_rate = LAYERS[0].3;

    for layer in LAYERS.iter().rev() {
        if h >= layer.0 {
            base_h = layer.0;
            base_p = layer.1;
            base_t = layer.2;
            lapse_rate = layer.3;
            break;
        }
    }

    // Calculate Temperature
    let temperature = base_t + lapse_rate * (h - base_h);

    // Calculate Pressure
    let pressure = if lapse_rate == 0.0 {
        // Isothermal layer
        base_p * (-G0 * M * (h - base_h) / (R * base_t)).exp()
    } else {
        // Gradient layer
        base_p * (base_t / temperature).powf(G0 * M / (R * lapse_rate))
    };

    // Calculate Density (Rho) using the Ideal Gas Law
    let density = (pressure * M) / (R * temperature);

    Atmosphere {
        pressure,
        density,
        temperature,
    }
}

fn thrust(input: &InputAxis, forward: &Dir3) -> Vec3 {
    let thrust_factor = 150_000.0;

    forward.as_vec3() * thrust_factor * input.throttle
}

fn induced_drag(lift_coeff: f32, rho: f32, speed: f32) -> f32 {
    let zero_lift_induced_drag_coeff = 0.0;
    let induced_drag_coeff =
        zero_lift_induced_drag_coeff + lift_coeff.powi(2) / std::f32::consts::PI * ASPECT_RATIO;
    let wingspan: f32 = 15.0;

    0.5 * rho * speed.powi(2) * induced_drag_coeff * wingspan.powi(2)
}

fn stabilise() -> Vec3 {
    Vec3::ZERO // TODO
}

fn lift(lift_coeff: f32, airspeed: f32, wing_area: f32, up: Dir3, rho: f32) -> Vec3 {
    let lift_force = lift_coeff * rho * (airspeed.powi(2) * 0.5) * wing_area;

    lift_force * up
}

pub fn canards_angle(
    aircraft: Single<'_, '_, (&Transform, &LinearVelocity), With<Aircraft>>,
    state: AircraftState,
) -> (f32, f32) {
    let velocity = aircraft.1.to_vec3a().to_vec3();
    let transform = aircraft.0;
    let sin = transform
        .forward()
        .cross(velocity)
        .dot(transform.right().as_vec3());
    let cos = transform.forward().dot(velocity);
    let alpha = -sin.atan2(cos);
    let alpha_deg = alpha.to_degrees();

    // Canards work the other around way when landing gear is deployed, maximising lift
    let factor = match state.landing_gear_deployed {
        false => 1.0,
        true => -1.0,
    };

    let canards_angle = if velocity.length() <= 1.0 {
        0.0
    } else {
        (factor * alpha_deg).clamp(-30.0, 50.0).to_radians()
    };

    // Port, Starboard
    (canards_angle, canards_angle)
}
