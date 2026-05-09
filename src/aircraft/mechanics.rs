pub mod aeroplane;

use crate::{
    aircraft::{Aircraft, AircraftState, AircraftTypes},
    input::InputAxis,
};
use avian3d::prelude::{Forces, SpatialQuery, WriteRigidBodyForces};
use bevy::prelude::*;

pub fn alpha_deg(velocity: &Vec3, transform: &GlobalTransform) -> f32 {
    let velocity = velocity.normalize_or_zero();
    let sin = transform
        .forward()
        .cross(velocity)
        .dot(transform.right().as_vec3());
    let cos = transform.forward().dot(velocity);
    let alpha = -sin.atan2(cos).to_degrees();

    alpha
}

pub fn mechanics(
    // mut force: Single<Forces, With<Aircraft>>,
    input: Res<InputAxis>,
    mut state: ResMut<AircraftState>,
    spatial_query: SpatialQuery,
    gizmos: Gizmos,
    mut aircraft: Single<(&GlobalTransform, Forces), With<Aircraft>>,
    time: Res<Time>,
) {
    let transform = &aircraft.0.clone();
    let mut force = &mut aircraft.1;
    match state.aircraft_type {
        AircraftTypes::Helicopter => {
            if state.engine.on {
                let thrust_factor = 120_000.;
                let thrust = transform.up() * thrust_factor * input.throttle;
                let torque = Vec3::new(input.pitch, input.yaw, input.roll);

                force.apply_force(thrust);
                force.apply_local_torque(torque * 500.0);
            }
        }
        AircraftTypes::Aeroplane => {
            aeroplane::mechanics(
                &mut state,
                &*transform,
                &mut force,
                spatial_query,
                gizmos,
                time,
            );
            aeroplane::fly_by_wire(&*input, &mut state, aircraft);
        }
    }
}

struct AircraftPhysicsConfig {
    pitch_point: Vec3,
    yaw_point: Vec3,
    roll_port_point: Vec3,
    roll_starboard_point: Vec3,
}

fn lift_coeff(alpha_deg: f32, potential_lift_factor: f32, vortex_lift_factor: f32) -> f32 {
    let alpha = alpha_deg.to_radians();

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
