use crate::aircraft::breeze::landing_gear::RIGHT_POS;
use crate::{aircraft::Aircraft, bevy_to_aerospace_coords};
use avian_fdm::prelude::AirfoilData;
use avian_fdm::{
    components::InducedDrag,
    prelude::{
        AeroCoeff, AeroZone, AeroZoneBundle, AircraftCoreBundle, AircraftGeometry,
        ControlSurfaceRole, EngineZone,
    },
    sourced,
};
use avian3d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::prelude::*;
use landing_gear::LEFT_POS;

pub mod fly_by_wire;
pub mod landing_gear;
pub mod mechanics;

// ── Aircraft reference constants ─────────────────────────────────────────────

/// JSBSim J3Cub reference wing area (m²): 178.50 ft² × 0.0929.
pub const WING_AREA_M2: Scalar = sourced!(
    49.0,
    "JSBSim:J3Cub.xml: wing_area 178.50 ft² × 0.0929 m²/ft²"
);

/// JSBSim J3Cub wingspan (m): 35.25 ft × 0.3048.
pub const WING_SPAN_M: Scalar = sourced!(10.0, "JSBSim:J3Cub.xml: wingspan 35.25 ft × 0.3048 m/ft");

/// JSBSim J3Cub mean aerodynamic chord (m): 5.25 ft × 0.3048.
pub const CHORD_M: Scalar = sourced!(1.600, "JSBSim:J3Cub.xml: chord 5.25 ft × 0.3048 m/ft");

/// Elevator chord (m): ~1.15 ft, trailing edge of h-stab.
const ELEVATOR_CHORD_M: Scalar = sourced!(
    0.35,
    "Geometry: J3 Cub elevator chord, approx 1.15 ft from type certificate drawings"
);

const ELEVATOR_AREA_M2: Scalar = 2.0;

/// Elevator CL per radian of deflection.
///
/// From JSBSim CM_de = -1.2004/rad. The whole-aircraft pitch moment from
/// elevator is: M = CM_de * delta * qbar * S_ref * c_ref.
///
/// With physical area: M = CL_elev * delta * qbar * S_elev * l_t.
/// So: CL_elev = CM_de * S_ref * c_ref / (S_elev * l_t)
///             = 1.2004 * 16.584 * 1.6 / (1.07 * 4.023) = 7.40/rad.
///
/// Negative: positive elevator (nose-up stick) produces downward tail force.
const ELEVATOR_CL_DELTA: Scalar = sourced!(
    -7.40,
    "Calibration: CL_elev = |CM_de| × S_ref × c / (S_elev × l_t) = 1.2004 × 16.584 × 1.6 / (1.07 × 4.023); negative for nose-up convention"
);

// ── Vertical tail geometry ───────────────────────────────────────────────────

/// Vertical fin height (m): from three-view drawings, root to tip.
const VFIN_HEIGHT_M: Scalar = sourced!(
    2.7,
    "Geometry: J3 Cub vertical fin height from three-view drawings"
);

/// Vertical fin mean chord (m): average of root (~0.65m) and tip (~0.35m).
const VFIN_MEAN_CHORD_M: Scalar = sourced!(
    2.5,
    "Geometry: J3 Cub vertical fin mean chord, (root 0.65 + tip 0.35) / 2"
);

/// Vertical fin planform area (m2): height * mean chord.
const VFIN_AREA_M2: Scalar = VFIN_HEIGHT_M * VFIN_MEAN_CHORD_M;

const VFIN_ARM_M: Scalar = 6.0;

/// Vertical fin CY per radian of sideslip.
///
/// From JSBSim CN_beta = 0.0602/rad. The whole-aircraft yaw moment from
/// sideslip is: N = CN_beta * beta * qbar * S_ref * b.
///
/// With physical fin area: N = CY_fin * beta * qbar * S_fin * x_arm.
/// So: CY_fin = CN_beta * S_ref * b / (S_fin * x_arm)
///            = 0.0602 * 16.584 * 10.742 / (0.425 * 3.6) = 7.01/rad.
///
/// Negative: positive beta (wind from right) produces leftward force at the
/// tail, restoring the nose toward the wind (weathercock stability).
const VFIN_CY_BETA: Scalar = sourced!(
    -7.01,
    "Calibration: CY_fin = CN_beta × S_ref × b / (S_fin × x_arm) = 0.0602 × 16.584 × 10.742 / (0.425 × 3.6); negative for restoring (weathercock) convention"
);

/// Rudder height (m): extends slightly beyond the fin (horn balance).
const RUDDER_HEIGHT_M: Scalar = sourced!(
    2.0,
    "Geometry: J3 Cub rudder height from three-view drawings"
);

/// Rudder mean chord (m): average of root (~0.45m) and tip (~0.30m).
const RUDDER_MEAN_CHORD_M: Scalar = sourced!(
    0.8,
    "Geometry: J3 Cub rudder mean chord, (root 0.45 + tip 0.30) / 2"
);

/// Rudder planform area (m2): height * mean chord.
const RUDDER_AREA_M2: Scalar = RUDDER_HEIGHT_M * RUDDER_MEAN_CHORD_M; // 0.356 m2

/// Rudder CY per radian of deflection.
///
/// From JSBSim CN_dr = -0.0565/rad. The whole-aircraft yaw moment from
/// rudder is: N = CN_dr * delta_r * qbar * S_ref * b.
///
/// With physical area: N = CY_rud * delta_r * qbar * S_rud * x_arm.
/// So: CY_rud = CN_dr * S_ref * b / (S_rud * x_arm)
///            = 0.0565 * 16.584 * 10.742 / (0.356 * 3.6) = 7.86/rad.
///
/// Negative: positive rudder (nose-right) produces leftward force at tail.
const RUDDER_CY_DELTA: Scalar = sourced!(
    -7.86,
    "Calibration: CY_rud = |CN_dr| × S_ref × b / (S_rud × x_arm) = 0.0565 × 16.584 × 10.742 / (0.356 × 3.6); negative for −Y force convention"
);

// ── Aileron geometry ─────────────────────────────────────────────────────────

/// Aileron span per side (m): occupies the outboard wing tip region.
const AILERON_SPAN_M: Scalar = sourced!(2.0, "Measured from model");

const AILERON_AREA_M2: Scalar = 1.44;

/// Aileron CL per radian of deflection.
///
/// From JSBSim Cl_da = 0.3498/rad. The whole-aircraft roll moment from
/// one aileron: M_roll = CL_ail * delta * qbar * S_ail * y_arm.
/// Two ailerons (differential): M_total = 2 * CL_ail * qbar * S_ail * y_arm * delta.
/// JSBSim: M_total = Cl_da * qbar * S_ref * b * delta.
///
/// So: CL_ail = Cl_da * S_ref * b / (2 * S_ail * y_arm)
///            = 0.3498 * 16.584 * 10.742 / (2 * 1.376 * 4.05) = 5.59/rad.
const AILERON_CL_DELTA: Scalar = sourced!(
    5.59,
    "Calibration: CL_ail = Cl_da × S_ref × b / (2 × S_ail × y_arm) = 0.3498 × 16.584 × 10.742 / (2 × 1.376 × 4.05)"
);

// ── Landing gear geometry ────────────────────────────────────────────────────

/// Gear leg frontal area (m2): exposed axle/bungee strut, approx 0.6m long * 0.04m diameter.
const GEAR_LEG_AREA_M2: Scalar = sourced!(
    0.024,
    "Geometry: J3 Cub gear leg frontal area, 0.6 m × 0.04 m exposed axle + bungee"
);

/// Gear leg drag coefficient (based on frontal area).
///
/// From JSBSim Drag_gear: each leg contributes CD = 0.001 against S_ref.
/// Physical: CD_leg = 0.001 * S_ref / S_leg = 0.001 * 16.584 / 0.024 = 0.691.
/// Typical for a partially faired strut (bare cylinder ~ 1.0-1.2).
const GEAR_LEG_CD: Scalar = sourced!(
    0.691,
    "Calibration: CD_leg = 0.001 × S_ref / S_leg = 0.001 × 16.584 / 0.024; partially faired strut"
);

/// Wheel frontal area (m2): circle with radius 0.15m (8-inch tyre).
const WHEEL_AREA_M2: Scalar = sourced!(
    0.0707,
    "Geometry: J3 Cub main wheel frontal area, pi × 0.15^2"
);

/// Wheel drag coefficient (based on frontal area).
///
/// From JSBSim Drag_gear: each wheel contributes CD = 0.001 against S_ref.
/// Physical: CD_wheel = 0.001 * S_ref / S_wheel = 0.001 * 16.584 / 0.0707 = 0.235.
/// Lower than a bare disc (~0.4-0.6) because JSBSim models it as residual drag.
const WHEEL_CD: Scalar = sourced!(
    0.235,
    "Calibration: CD_wheel = 0.001 × S_ref / S_wheel = 0.001 × 16.584 / 0.0707; JSBSim residual"
);

/// Wing aerodynamic-centre x-offset from entity root (m).
const WING_AC_X: Scalar = -3.5;

/// Wing height above CG in body frame (m, negative = up since +Z = down).
const WING_Z: Scalar = sourced!(
    0.0,
    "JSBSim:J3Cub_FlightGear.xml: CG z = −23.23 in; wing datum z = 0 -> 23.23 in = 0.590 m"
);

/// Geometric dihedral of each wing panel (radians).
const WING_DIHEDRAL_RAD: Scalar = sourced!(
    -0.02,
    "Geometry: J3 Cub wing dihedral approximately 4 deg; provides Cl_beta lateral stability"
);

const ENGINE_LENGTH: f32 = sourced!(
    3.538,
    "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Length%3A%20353.8%C2%A0cm%20(139.3%C2%A0in)"
);
const ENGINE_RADIUS: f32 = sourced!(
    0.696 / 2.0,
    "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Diameter%3A%2069.6%C2%A0cm%20(27.4%C2%A0in)"
);

use avian_fdm::airfoil::foil_tools::parse_foil_tools_csv;

pub fn airfoil() -> AirfoilData {
    let csv: &str = include_str!("../../assets/aircraft/breeze/c-f3/naca2408_polars.csv");
    parse_foil_tools_csv(csv)
        .expect("embedded profile to parse cleanly")
        .ncrit9
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Spawn a complete aircraft with all child [`AeroZone`] entities.
///
/// Returns the root entity ID. The aircraft root is spawned at `transform`
/// (typically over the runway at some altitude). Add your own input system that
/// writes to [`avian_fdm::components::ControlInputs`] on the root entity.
pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    asset_server: Res<AssetServer>,
) -> Entity {
    use avian_fdm::components::GizmoShape;

    const PATH: &str = "aircraft/breeze/c-f3/model.gltf";

    let root = commands
        .spawn((
            SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(PATH))),
            breeze_core_bundle(transform),
            // Lift-induced drag: J3Cub has a high-wing strut-braced layout.
            // e = 0.94 from JSBSim: CD_i = CL² × 0.0485, so e = 1/(π × 0.0485 × AR=6.956)
            InducedDrag {
                oswald_factor: sourced!(
                    0.94,
                    "JSBSim:J3Cub.xml: CD_i = CL²×0.0485 -> e = 1/(π×0.0485×AR=6.956) ≈ 0.94"
                ),
            },
            Aircraft,
            // No LodDamping. Roll/pitch/yaw damping emerges from zone geometry.
        )).observe(crate::data_from_gltf::load)
        .with_children(|parent| {
            // ── Fuselage aft (tail boom) ─────────────────────────────────────
            parent.spawn((
                AeroZoneBundle {
                    zone: AeroZone {
                        cl: AeroCoeff::Scalar(0.0),
                        cd: AeroCoeff::Scalar(0.0),
                        ..Default::default()
                    },
                    collider: Collider::cuboid(2.05, 0.40, 0.35),
                    transform: Transform::from_xyz(1.82, 0.0, 0.0),
                    global_transform: GlobalTransform::default(),
                },
                Mass(sourced!(1000.0, "Estimate")),
                GizmoShape::Box { x: 2.05, y: 0.40, z: 0.35 }
            ));

            const ROOT_X_M: f32 = 4.0;
            const ROOT_Y_M: f32 = 1.2;
            const ROOT_OFFSET: f32 = 0.5;

            // ── Left wing ───────────────────────────────────────────────────
            // Thin collider (z=0.02 m). See module docs on hybrid approach.
            parent.spawn((wing_zone(
                "L-root", WING_AC_X + ROOT_OFFSET, WING_AC_X, -ROOT_Y_M, 0.175,
                airfoil(),
                Collider::cuboid(ROOT_X_M, 1.88, 0.2),
                ColliderDensity(sourced!(585.0, "Inertia-calibrated: uniform wing density; total wing mass ~80 kg for Ixx=729")),
            ), GizmoShape::Box { x: ROOT_X_M, y: 1.88, z: 0.2 }));
            parent.spawn((wing_zone(
                "L-mid", WING_AC_X, WING_AC_X, -2.82, 0.175,
                airfoil(),
                Collider::cuboid(0.80, 1.88, 0.02),
                ColliderDensity(sourced!(585.0, "Inertia-calibrated: uniform wing density; total wing mass ~80 kg for Ixx=729")),
            ), GizmoShape::Box { x: 0.80, y: 1.88, z: 0.02 }));

            const TIP_X_M: f32 = -4.5;

            // Tip strip (LE portion of chord, outboard alongside the aileron).
            // Entity at geometric center of the strip (0.075) for correct
            // collider position; ac_offset inside AeroZone shifts the force
            // application point to WING_AC_X (25% of the full wing chord).
            parent.spawn((wing_zone(
                "L-tip", TIP_X_M, WING_AC_X, -4.19, 0.150,
                airfoil(),
                Collider::cuboid(0.45, 0.86, 0.02),
                ColliderDensity(585.0),
            ), GizmoShape::Box { x: 0.45, y: 0.86, z: 0.02 }));

            // ── Right wing ───────────────────────────────────────────────────
            parent.spawn((wing_zone(
                "R-root", WING_AC_X + ROOT_OFFSET, WING_AC_X, ROOT_Y_M, 0.175,
                airfoil(),
                Collider::cuboid(ROOT_X_M, 1.88, 0.2),
                ColliderDensity(585.0),
            ), GizmoShape::Box { x: ROOT_X_M, y: 1.88, z: 0.2 }));
            parent.spawn((wing_zone(
                "R-mid", WING_AC_X, WING_AC_X, 2.82, 0.175,
                airfoil(),
                Collider::cuboid(0.80, 1.88, 0.02),
                ColliderDensity(585.0),
            ), GizmoShape::Box { x: 0.80, y: 1.88, z: 0.02 }));
            parent.spawn((wing_zone(
                "R-tip", TIP_X_M, WING_AC_X, 4.19, 0.150,
                airfoil(),
                Collider::cuboid(0.45, 0.86, 0.02),
                ColliderDensity(585.0),
            ), GizmoShape::Box { x: 0.45, y: 0.86, z: 0.02 }));

            // ── Ailerons ─────────────────────────────────────────────────────
            const AILERON_LENGTH: f32 = 0.72;

            // Trailing-edge strip, outboard: tiled behind tip front and
            // spanning from mid panel end (3.76) to wingtip (5.37).
            // Aileron span = 0.75m per side, center at 3.76 + 0.86 + 0.75/2 = 4.995
            // WRONG. That places them outside the wing. The aileron sits at
            // the SAME spanwise station as the tip, occupying the TE strip:
            // tip_main covers y = [3.76, 4.62], aileron covers y = [3.87, 4.62]
            // sharing the outboard span. Actually the tip+aileron tile the
            // outboard region: tip is the LE strip, aileron is the TE strip,
            // both at the SAME Y range.
            // Center Y = same as tip = 4.19.
            parent.spawn((aileron_zone(
                "L-aileron", -4.19,
                ControlSurfaceRole::AileronLeft,
                Collider::cuboid(AILERON_LENGTH, AILERON_SPAN_M, 0.02),
                ColliderDensity(sourced!(585.0, "Inertia-calibrated: same as wing panels (control surface + structure)")),
            ), GizmoShape::Box { x: AILERON_LENGTH, y: AILERON_SPAN_M, z: 0.02 }));
            parent.spawn((aileron_zone(
                "R-aileron", 4.19,
                ControlSurfaceRole::AileronRight,
                Collider::cuboid(AILERON_LENGTH, AILERON_SPAN_M, 0.02),
                ColliderDensity(sourced!(585.0, "Inertia-calibrated: same as wing panels (control surface + structure)")),
            ), GizmoShape::Box { x: AILERON_LENGTH, y:AILERON_SPAN_M, z: 0.02 }));

            // ── Landing gear legs ────────────────────────────────────────────
            for (sign, name) in [(-1.0_f32, "L-gear"), (1.0, "R-gear")] {
                let top = Vec3::new(0.50, 0.15 * sign, 0.35);
                let bottom = Vec3::new(0.50, 0.55 * sign, 0.90);
                let mid = (top + bottom) * 0.5;
                let dir = bottom - top;
                let length = dir.length();
                let rot = Quat::from_rotation_arc(Vec3::X, dir.normalize());
                let half = length * 0.5;
                parent.spawn((
                    Name::new(name),
                    AeroZoneBundle {
                        zone: AeroZone {
                            cl: AeroCoeff::Scalar(0.0),
                            cd: AeroCoeff::Scalar(GEAR_LEG_CD),
                            area_m2: GEAR_LEG_AREA_M2,
                            ..Default::default()
                        },
                        collider: Collider::cuboid(length as Scalar, 0.04, 0.04),
                        transform: Transform::from_translation(mid).with_rotation(rot),
                        global_transform: GlobalTransform::default(),
                    },
                    ColliderDensity(sourced!(7800.0, "Literature: steel axle/bungee landing gear; standard mild steel density")),
                    GizmoShape::Strut {
                        start: Vec3::new(-half, 0.0, 0.0),
                        end: Vec3::new(half, 0.0, 0.0),
                    },
                ));
            }

            // ── Main wheels ──────────────────────────────────────────────────
            for (position, name) in [(bevy_to_aerospace_coords() * LEFT_POS, "L-wheel"), (bevy_to_aerospace_coords() * RIGHT_POS, "R-wheel")] {
                parent.spawn((
                    Name::new(name),
                    AeroZoneBundle {
                        zone: AeroZone {
                            cl: AeroCoeff::Scalar(0.0),
                            cd: AeroCoeff::Scalar(WHEEL_CD),
                            area_m2: WHEEL_AREA_M2,
                            ..Default::default()
                        },
                        collider: Collider::cuboid(0.30, 0.10, 0.30),
                        transform: Transform::from_translation(position),
                        global_transform: GlobalTransform::default(),
                    },
                    ColliderDensity(sourced!(1200.0, "Estimate: 8-ply tyre + aluminium rim; composite density ≈ rubber 1100 + Al 2700")),
                    // Wheels roll around Y (spanwise axis); radius 0.15 m, width 0.10 m.
                    GizmoShape::Cylinder { radius: 0.15, length: 0.10, axis: Vec3::Y },
                ));
            }

            // ── Elevator ──────────────────────────────────────────────────────
            parent.spawn((
                Name::new("elevator-L"),
                elevator_zone(
                    Collider::cuboid(0.75, 2.0, 0.02),
                    ColliderDensity(sourced!(100.0, "Inertia-calibrated: elevator lighter than h-stab; ~0.7 kg total")),
                    -2.2
                ),
                GizmoShape::Box { x: 0.75, y: 2.0, z: 0.02 }
            ));
            parent.spawn((
                Name::new("elevator-R"),
                elevator_zone(
                    Collider::cuboid(0.75, 2.0, 0.02),
                    ColliderDensity(sourced!(100.0, "Inertia-calibrated: elevator lighter than h-stab; ~0.7 kg total")),
                    2.2
                ),
                GizmoShape::Box { x: 0.75, y: 2.0, z: 0.02 },
            ));

            // ── Vertical fin ──────────────────────────────────────────────────
            parent.spawn((
                Name::new("vertical fin"),
                vtail_zone(
                    Collider::cuboid(VFIN_MEAN_CHORD_M, 0.10, VFIN_HEIGHT_M),
                    ColliderDensity(sourced!(30.0, "Inertia-calibrated: vertical fin fabric/wood structure; ~1.7 kg")),
                ),
                GizmoShape::Box { x: VFIN_MEAN_CHORD_M, y: 0.1, z: VFIN_HEIGHT_M }
            ));

            // ── Rudder ────────────────────────────────────────────────────────
            // LE is the hinge line (matches vtail TE at x = −3.825 body).
            // Real J3 Cub: root chord ~0.45m, tip ~0.30m, height ~0.95m.
            parent.spawn((rudder_zone(
                    Collider::cuboid(RUDDER_MEAN_CHORD_M, 0.07, RUDDER_HEIGHT_M),
                    ColliderDensity(sourced!(105.0, "Estimate")),
                ),
                Name::new("rudder"),
                GizmoShape::Box { x: RUDDER_MEAN_CHORD_M, y: 0.07, z: RUDDER_HEIGHT_M }
            ));

            // ── Engines ───────────────────────────────────────────────────────
            parent.spawn((
                Name::new("engine-L"),
                engine_zone(
                    Collider::cuboid(ENGINE_LENGTH, ENGINE_RADIUS * 2.0, ENGINE_RADIUS * 2.0),
                    Mass(sourced!(897.0, "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Dry%20weight%3A%20897%C2%A0kg%20(1%2C978%C2%A0lb)")),
                    -0.6
                ),
                GizmoShape::Cylinder { radius: ENGINE_RADIUS, length: ENGINE_LENGTH, axis: Vec3::X },
            ));
            parent.spawn((
                Name::new("engine-R"),
                engine_zone(
                    Collider::cuboid(ENGINE_LENGTH, ENGINE_RADIUS * 2.0, ENGINE_RADIUS * 2.0),
                    Mass(sourced!(897.0, "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Dry%20weight%3A%20897%C2%A0kg%20(1%2C978%C2%A0lb)")),
                    0.6
                ),
                GizmoShape::Cylinder { radius: ENGINE_RADIUS, length: ENGINE_LENGTH, axis: Vec3::X },
            ));
        })
        .id();

    root
}

/// Core [`AircraftCoreBundle`] for the J-3 Cub root entity.
///
/// Mass, CoG, and inertia are computed by Avian from child zone colliders.
///
/// Pair with [`InducedDrag`] (already included by [`spawn`]) for lift-induced
/// drag.  No [`LodDamping`](avian_fdm::components::LodDamping). Roll/pitch/yaw
/// damping emerges from per-zone local α/β physics.
pub fn breeze_core_bundle(transform: Transform) -> impl Bundle {
    (AircraftCoreBundle {
        geometry: AircraftGeometry {
            wing_area_m2: WING_AREA_M2,
            wing_span_m: WING_SPAN_M,
            chord_m: CHORD_M,
        },
        rigid_body: RigidBody::Dynamic,
        transform,
        ..Default::default()
    },)
}

// ── Zone builder functions (pub for testing / custom assemblies) ──────────────

/// One wing panel zone at position (`x_m`, `y_m`, on the dihedral plane).
///
/// `x_m` is the entity and collider center along the chord axis (physical
/// position, used for mass distribution). `ac_x_m` is the aerodynamic center
/// where lift forces are applied; for all wing panels this should be
/// `WING_AC_X` regardless of how the chord is partitioned. When
/// `ac_x_m == x_m` the `ac_offset` inside [`AeroZone`] is zero.
///
/// `fraction` is the fraction of the total wing area this panel represents.
/// The panel's aerodynamic area is `fraction * WING_AREA_M2` and the CL/CD
/// tables are taken from `airfoil` (unscaled).
#[allow(clippy::too_many_arguments)]
pub fn wing_zone(
    name: &'static str,
    x_m: Scalar,
    ac_x_m: Scalar,
    y_m: Scalar,
    fraction: Scalar,
    airfoil: AirfoilData,
    collider: Collider,
    density: ColliderDensity,
) -> impl Bundle {
    let ac_offset = Vec3::new((ac_x_m - x_m) as f32, 0.0, 0.0);
    let z_m = WING_Z - y_m.abs() * WING_DIHEDRAL_RAD.sin();
    let dihedral_rot = Quat::from_rotation_x(-(WING_DIHEDRAL_RAD * y_m.signum()) as f32);
    (
        Name::new(name),
        AeroZoneBundle {
            zone: AeroZone {
                cl: airfoil.cl,
                cd: airfoil.cd,
                ac_offset,
                area_m2: fraction * WING_AREA_M2,
                chord_m: CHORD_M,
                ..Default::default()
            }
            .with_post_stall_extension(),
            collider,
            transform: Transform::from_xyz(x_m as f32, y_m as f32, z_m as f32)
                .with_rotation(dihedral_rot),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// Aileron zone at lateral offset `y_m` with the given control role.
pub fn aileron_zone(
    name: &'static str,
    y_m: Scalar,
    role: ControlSurfaceRole,
    collider: Collider,
    density: ColliderDensity,
) -> impl Bundle {
    let aileron_x = (WING_AC_X - 2.0) as f32;
    let z_m = WING_Z - y_m.abs() * WING_DIHEDRAL_RAD.sin();
    let dihedral_rot = Quat::from_rotation_x(-(WING_DIHEDRAL_RAD * y_m.signum()) as f32);
    (
        Name::new(name),
        AeroZoneBundle {
            zone: AeroZone {
                cl: AeroCoeff::Scalar(AILERON_CL_DELTA),
                cd: AeroCoeff::Scalar(0.0), // included in wing CD_basic
                control_role: Some(role),
                area_m2: AILERON_AREA_M2,
                chord_m: CHORD_M,
                ..Default::default()
            },
            collider,
            transform: Transform::from_xyz(aileron_x, y_m as f32, z_m as f32)
                .with_rotation(dihedral_rot),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// Horizontal stabiliser zone: provides pitch stability via tail-arm moment.
///
/// Uses the physical h-stab planform area (HSTAB_AREA_M2 = 1.86 m2) and an
/// effective lift curve slope (HSTAB_CL_ALPHA = 7.2/rad) calibrated to match
/// JSBSim CM_alpha. The CL vs alpha relationship is linear (symmetric airfoil):
///   CL = HSTAB_CL_ALPHA * alpha
///
/// At alpha > 0 (nose up), the h-stab produces positive CL (upward force),
/// which at the aft arm creates a nose-down restoring moment.
///
/// The table is extended to +/-180 deg via Viterna-Corrigan post-stall model,
/// so the h-stab stalls realistically at high alpha and produces flat-plate
/// drag when broadside to the wind. This prevents unrealistic pitch-locking
/// during tumbles and deep stalls.

/// Elevator zone: pitch control surface.
///
/// Uses the physical elevator area (ELEVATOR_AREA_M2 = 1.07 m2) and an
/// effective CL per radian of deflection (ELEVATOR_CL_DELTA = -7.40/rad)
/// calibrated to match JSBSim CM_de.
///
/// Negative CL means: positive elevator (nose-up stick input) produces downward
/// force at the tail, creating a nose-up pitch moment via the tail arm.
pub fn elevator_zone(collider: Collider, density: ColliderDensity, y_m: Scalar) -> impl Bundle {
    let elevator_x = (WING_AC_X - 2.0) as f32;
    let z_m = WING_Z - y_m.abs() * WING_DIHEDRAL_RAD.sin();

    (
        AeroZoneBundle {
            zone: AeroZone {
                cl: AeroCoeff::Scalar(ELEVATOR_CL_DELTA),
                cd: AeroCoeff::Scalar(0.0),
                control_role: Some(ControlSurfaceRole::Elevator),
                area_m2: ELEVATOR_AREA_M2,
                chord_m: ELEVATOR_CHORD_M,
                ..Default::default()
            },
            collider,
            transform: Transform::from_xyz(elevator_x, y_m, z_m),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// Vertical tail zone: structural mass and weathercock stability.
///
/// Uses the physical fin planform area (VFIN_AREA_M2 = 0.425 m2) and an
/// effective CY per radian of sideslip (VFIN_CY_BETA = -7.01/rad) calibrated
/// to match JSBSim CN_beta.
///
/// Negative CY_beta: positive sideslip (wind from right) produces a leftward
/// force at the aft tail, generating a restoring (nose-right) yaw moment.
///
/// The CY table is extended to +/-180 deg via Viterna-Corrigan so the fin
/// stalls realistically in deep sideslip and does not lock the aircraft
/// into an unrealistic yaw pattern during tumbles.
pub fn vtail_zone(collider: Collider, density: ColliderDensity) -> impl Bundle {
    (
        AeroZoneBundle {
            zone: AeroZone {
                cl: AeroCoeff::Scalar(0.0),
                cd: AeroCoeff::Scalar(sourced!(
                    0.01,
                    "Estimate: symmetric airfoil profile drag at low beta"
                )),
                cy: AeroCoeff::Table1D {
                    breakpoints: vec![-avian3d::math::FRAC_PI_2, 0.0, avian3d::math::FRAC_PI_2],
                    values: vec![
                        -VFIN_CY_BETA * avian3d::math::FRAC_PI_2,
                        0.0,
                        VFIN_CY_BETA * avian3d::math::FRAC_PI_2,
                    ],
                },
                area_m2: VFIN_AREA_M2,
                chord_m: VFIN_MEAN_CHORD_M,
                ..Default::default()
            }
            .with_post_stall_extension(),
            collider,
            transform: Transform::from_xyz(-(VFIN_ARM_M as f32), 0.0, -2.35),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// Rudder zone: yaw control surface.
///
/// Uses the physical rudder planform area (RUDDER_AREA_M2 = 0.356 m2) and an
/// effective CY per radian of deflection (RUDDER_CY_DELTA = -7.86/rad)
/// calibrated to match JSBSim CN_dr.
///
/// Negative CY: positive rudder (nose-right) produces leftward force at the
/// tail, generating positive (nose-right) yaw torque.
pub fn rudder_zone(collider: Collider, density: ColliderDensity) -> impl Bundle {
    (
        AeroZoneBundle {
            zone: AeroZone {
                cl: AeroCoeff::Scalar(0.0),
                cd: AeroCoeff::Scalar(0.0),
                cy: AeroCoeff::Scalar(RUDDER_CY_DELTA),
                control_role: Some(ControlSurfaceRole::Rudder),
                area_m2: RUDDER_AREA_M2,
                chord_m: RUDDER_MEAN_CHORD_M,
                ..Default::default()
            },
            collider,
            // Rudder LE is at the fin TE. Fin center at -VFIN_ARM_M, fin extends
            // VFIN_MEAN_CHORD_M/2 aft, so rudder LE = -(VFIN_ARM_M + VFIN_MEAN_CHORD_M/2).
            // Rudder center = rudder LE - RUDDER_MEAN_CHORD_M/2.
            transform: Transform::from_xyz(
                -((VFIN_ARM_M + VFIN_MEAN_CHORD_M / 2.0 + RUDDER_MEAN_CHORD_M / 2.0) as f32),
                0.0,
                -2.0,
            ),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

pub fn engine_zone(collider: Collider, mass: Mass, y_m: f32) -> impl Bundle {
    (
        EngineZone {
            max_thrust_n: sourced!(
                7_330.0 * 9.80665,
                "https://wiki.warthunder.com/unit/rafale_c_f3: Afterburner"
            ),
            throttle_curve: sourced!(
                vec![[0.0, 0.0], [0.5, 0.5], [0.9, 0.668485675], [1.0, 1.0]],
                "https://wiki.warthunder.com/unit/rafale_c_f3: Afterburner at 100%"
            ),
            thrust_axis_body: Vector::X, // +X = forward
            zero_thrust_speed_ms: Some(sourced!(80.0, "Estimate")),
        },
        collider,
        mass,
        Transform::from_xyz(-5.2, y_m, -0.4),
        GlobalTransform::default(),
    )
}
