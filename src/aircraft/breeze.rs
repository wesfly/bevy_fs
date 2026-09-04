use std::f32::consts::FRAC_PI_2;

use super::{aircraft::Aircraft, airfoils::ag47ct02r};
use crate::aircraft::{ControlSurfaces, airfoils::naca0010};
use avian_fdm::{
    components::{GizmoShape, InducedDrag},
    prelude::{
        AeroCoeff, AeroZone, AeroZoneBundle, AircraftCoreBundle, AircraftGeometry, AirfoilData,
        ControlSurfaceRole, EngineZone,
    },
    sourced,
};
use avian3d::{
    math::{Scalar, Vector},
    prelude::*,
};
use bevy::{math::DVec3, prelude::*};
use big_space::prelude::*;

pub mod fly_by_wire;
pub mod landing_gear;
pub mod mechanics;

// ── Aircraft reference constants ─────────────────────────────────────────────

pub const WING_AREA_M2: Scalar = sourced!(
    45.7,
    "https://en.wikipedia.org/wiki/Dassault_Rafale#:~:text=Wing%20area%3A%2045.7%C2%A0m2%20(492%C2%A0sq%C2%A0ft)"
);

pub const WING_SPAN_M: Scalar = sourced!(
    10.90,
    "https://en.wikipedia.org/wiki/Dassault_Rafale#:~:text=Wingspan%3A%2010.90%C2%A0m%20(35%C2%A0ft%209%C2%A0in)"
);

/// Mean aerodynamic chord (m)
pub const CHORD_M: Scalar = sourced!(5.0, "Estimate");

/// Elevator chord (m)
const ELEVATOR_CHORD_M: Scalar = sourced!(0.93, "Measured from model");

const ELEVATOR_AREA_M2: Scalar = sourced!(1.805, "Measured from model");

/// Elevator CL per radian of deflection.
const ELEVATOR_CL_DELTA: Scalar = sourced!(-7.0, "Estimate");

// ── Vertical tail geometry ───────────────────────────────────────────────────

/// Vertical fin height (m)
const VFIN_HEIGHT_M: Scalar = sourced!(2.7, "Measured from model");

/// Vertical fin mean chord (m)
const VFIN_MEAN_CHORD_M: Scalar = sourced!(2.5, "Measured from model");

/// Vertical fin planform area (m2): height * mean chord.
const VFIN_AREA_M2: Scalar = VFIN_HEIGHT_M * VFIN_MEAN_CHORD_M;

const VFIN_ARM_M: Scalar = 6.0;

/// Vertical fin CY per radian of sideslip.
const VFIN_CY_BETA: Scalar = sourced!(-5.0, "Estimate");

/// Rudder height (m): extends slightly beyond the fin (horn balance).
const RUDDER_HEIGHT_M: Scalar = sourced!(2.0, "Measured from model");

/// Rudder mean chord (m)
const RUDDER_MEAN_CHORD_M: Scalar = sourced!(0.8, "Measured from model");

/// Rudder planform area (m2): height * mean chord.
const RUDDER_AREA_M2: Scalar = RUDDER_HEIGHT_M * RUDDER_MEAN_CHORD_M;

/// Rudder CY per radian of deflection.
const RUDDER_CY_DELTA: Scalar = sourced!(-7.0, "Estimate");

// ── Aileron geometry ─────────────────────────────────────────────────────────

/// Aileron span per side (m): occupies the outboard wing tip region.
const AILERON_SPAN_M: Scalar = sourced!(2.0, "Measured from model");

const AILERON_AREA_M2: Scalar = 1.44;

/// Aileron CL per radian of deflection.
const AILERON_CL_DELTA: Scalar = sourced!(6.0, "Estimate");

/// Wing aerodynamic-centre x-offset from entity root (m).
const WING_AC_X: Scalar = -2.8;

/// Wing height above CG in body frame (m, negative = up since +Z = down).
const WING_Z: Scalar = sourced!(-0.3, "Estimate");

/// Geometric dihedral of each wing panel (radians).
const WING_DIHEDRAL_RAD: Scalar = sourced!(-4.0_f64.to_radians(), "Measured from model");

const ENGINE_LENGTH: f64 = sourced!(
    3.538,
    "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Length%3A%20353.8%C2%A0cm%20(139.3%C2%A0in)"
);
const ENGINE_RADIUS: f64 = sourced!(
    0.696 / 2.0,
    "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Diameter%3A%2069.6%C2%A0cm%20(27.4%C2%A0in)"
);

/// Spawn a complete aircraft with all child [`AeroZone`] entities.
///
/// Returns the root entity ID. The aircraft root is spawned at `transform`
/// (typically over the runway at some altitude). Add your own input system that
/// writes to [`avian_fdm::components::ControlInputs`] on the root entity.
pub fn spawn(
    commands: &mut Commands,
    transform: Transform,
    asset_server: Res<AssetServer>,
    rotation: Rotation,
    initial_velocity: Scalar,
    cell_coord: CellCoord,
    parent_id: Entity,
    abs_position: DVec3,
) -> Entity {
    const PATH: &str = "aircraft/breeze/c-f3/model.gltf";

    commands.spawn((
            cell_coord,
            breeze_core_bundle(transform, rotation * DVec3::X * initial_velocity),
            InducedDrag {
                oswald_factor: 0.94,
            },
            Aircraft,
            Position(abs_position),
            ChildOf(parent_id),
            rotation,
            // No LodDamping. Roll/pitch/yaw damping emerges from zone geometry.
        ))
            .with_children(|parent| {
                // 3D model
                parent.spawn((
                    WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(PATH))),
                    Transform::from_rotation(Quat::from_rotation_x(-FRAC_PI_2)),
                ));
                //==== front avionics/cabin fuselage section ====
                parent.spawn((
                    AeroZoneBundle {
                        zone: AeroZone {
                            cl: AeroCoeff::Scalar(0.0),
                            cd: AeroCoeff::Scalar(0.0),
                            ..Default::default()
                        },
                        collider: Collider::cuboid(2.05, 0.40, 0.35),
                        transform: Transform::from_xyz(4.0, 0.0, 0.0),
                        global_transform: GlobalTransform::default(),
                    },
                    Mass(1500.0),
                    GizmoShape::Box { x: 2.05, y: 0.40, z: 0.35 }
                ));

                //==== mid fuselage section ====
                parent.spawn((
                    AeroZoneBundle {
                        zone: AeroZone {
                            cl: AeroCoeff::Scalar(0.0),
                            cd: AeroCoeff::Scalar(0.0),
                            ..Default::default()
                        },
                        collider: Collider::cuboid(4.0, 2.0, 0.8),
                        transform: Transform::from_xyz(-2.0, 0.0, 0.0),
                        global_transform: GlobalTransform::default(),
                    },
                    Mass(5500.0),
                    GizmoShape::Box { x: 4.0, y: 2.0, z: 0.8 }
                ));

                const ROOT_X_M: f64 = 4.0;
                const ROOT_Y_M: f64 = 1.2;
                const ROOT_OFFSET: f64 = 0.5;

                const TIP_X_M: f64 = -4.5;

                // ── Left wing ───────────────────────────────────────────────────
                parent.spawn((wing_zone(
                    "L-root", WING_AC_X + ROOT_OFFSET, WING_AC_X, -ROOT_Y_M, 0.25,
                    ag47ct02r(),
                    Collider::cuboid(ROOT_X_M, 1.88, 0.2),
                    Mass(100.0),
                ), GizmoShape::Box { x: ROOT_X_M as f32, y: 1.88, z: 0.2 }));
                parent.spawn((wing_zone(
                    "L-mid", WING_AC_X, WING_AC_X, -2.82, 0.15,
                    ag47ct02r(),
                    Collider::cuboid(2.80, 1.88, 0.2),
                    Mass(80.0),
                ), GizmoShape::Box { x: 2.80, y: 1.88, z: 0.2 }));

                parent.spawn((wing_zone(
                    "L-tip", TIP_X_M, WING_AC_X, -4.19, 0.1,
                    ag47ct02r(),
                    Collider::cuboid(0.45, 0.86, 0.02),
                    Mass(50.0),
                ), GizmoShape::Box { x: 0.45, y: 0.86, z: 0.02 }));

                // ── Right wing ───────────────────────────────────────────────────
                parent.spawn((wing_zone(
                    "R-root", WING_AC_X + ROOT_OFFSET, WING_AC_X, ROOT_Y_M, 0.25,
                    ag47ct02r(),
                    Collider::cuboid(ROOT_X_M, 1.88, 0.2),
                    Mass(100.0),
                ), GizmoShape::Box { x: ROOT_X_M as f32, y: 1.88, z: 0.2 }));
                parent.spawn((wing_zone(
                    "R-mid", WING_AC_X, WING_AC_X, 2.82, 0.15,
                    ag47ct02r(),
                    Collider::cuboid(2.80, 1.88, 0.2),
                    Mass(80.0),
                ), GizmoShape::Box { x: 2.80, y: 1.88, z: 0.2 }));
                parent.spawn((wing_zone(
                    "R-tip", TIP_X_M, WING_AC_X, 4.4, 0.1,
                    ag47ct02r(),
                    Collider::cuboid(1.0, 1.5, 0.1),
                    Mass(50.0),
                ), GizmoShape::Box { x: 1.0, y: 1.5, z: 0.1 }));

                //=== canards ====
                const CANARD_AREA_M2: f64 = 1.0;

                parent.spawn((
                    Name::new("L-canard"),
                    ControlSurfaces::CanardPort,
                    AeroZoneBundle {
                        zone: AeroZone {
                            cl: naca0010().cl,
                            cd: naca0010().cd,
                            area_m2: CANARD_AREA_M2,
                            chord_m: CHORD_M,
                            ac_offset: Vec3::new(0.0, 0.0, 0.0),
                            ..Default::default()
                        }
                        .with_post_stall_extension(),
                        collider: Collider::cuboid(1.0, 1.0, 0.1),
                        transform: Transform::from_xyz(1.0, -1.8, -0.6),
                        global_transform: GlobalTransform::default(),
                    },
                    Mass(200.0),
                    GizmoShape::Box { x: 1.0, y: 1.0, z: 0.1 }
                ));

                parent.spawn((
                    Name::new("R-canard"),
                    ControlSurfaces::CanardStarboard,
                    AeroZoneBundle {
                        zone: AeroZone {
                            cl: naca0010().cl,
                            cd: naca0010().cd,
                            area_m2: CANARD_AREA_M2,
                            chord_m: CHORD_M,
                            ac_offset: Vec3::new(0.0, 0.0, 0.0),
                            ..Default::default()
                        }
                        .with_post_stall_extension(),
                        collider: Collider::cuboid(1.0, 1.0, 0.1),
                        transform: Transform::from_xyz(0.8, 1.8, -0.6),
                        global_transform: GlobalTransform::default(),
                    },
                    Mass(200.0),
                    GizmoShape::Box { x: 1.0, y: 1.0, z: 0.1 }
                ));

                // ── Ailerons ─────────────────────────────────────────────────────
                const AILERON_LENGTH: f64 = 0.72;

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
                ), GizmoShape::Box { x: AILERON_LENGTH as f32, y: AILERON_SPAN_M as f32, z: 0.02 }));
                parent.spawn((aileron_zone(
                    "R-aileron", 4.19,
                    ControlSurfaceRole::AileronRight,
                    Collider::cuboid(AILERON_LENGTH, AILERON_SPAN_M, 0.02),
                    ColliderDensity(sourced!(585.0, "Inertia-calibrated: same as wing panels (control surface + structure)")),
                ), GizmoShape::Box { x: AILERON_LENGTH as f32, y:AILERON_SPAN_M as f32, z: 0.02 }));

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
                    GizmoShape::Box { x: VFIN_MEAN_CHORD_M as f32, y: 0.1, z: VFIN_HEIGHT_M as f32 }
                ));

                // ── Rudder ────────────────────────────────────────────────────────
                parent.spawn((rudder_zone(
                        Collider::cuboid(RUDDER_MEAN_CHORD_M, 0.07, RUDDER_HEIGHT_M),
                        ColliderDensity(sourced!(105.0, "Estimate")),
                    ),
                    Name::new("rudder"),
                    GizmoShape::Box { x: RUDDER_MEAN_CHORD_M as f32, y: 0.07, z: RUDDER_HEIGHT_M as f32 }
                ));

                // ── Engines ───────────────────────────────────────────────────────
                parent.spawn((
                    Name::new("engine-L"),
                    engine_zone(
                        Collider::cuboid(ENGINE_LENGTH, ENGINE_RADIUS * 2.0, ENGINE_RADIUS * 2.0),
                        Mass(sourced!(897.0, "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Dry%20weight%3A%20897%C2%A0kg%20(1%2C978%C2%A0lb)")),
                        -0.6
                    ),
                    GizmoShape::Cylinder { radius: ENGINE_RADIUS as f32, length: ENGINE_LENGTH as f32, axis: Vec3::X },
                ));
                parent.spawn((
                    Name::new("engine-R"),
                    engine_zone(
                        Collider::cuboid(ENGINE_LENGTH, ENGINE_RADIUS * 2.0, ENGINE_RADIUS * 2.0),
                        Mass(sourced!(897.0, "https://en.wikipedia.org/wiki/Snecma_M88#:~:text=Dry%20weight%3A%20897%C2%A0kg%20(1%2C978%C2%A0lb)")),
                        0.6
                    ),
                    GizmoShape::Cylinder { radius: ENGINE_RADIUS as f32, length: ENGINE_LENGTH as f32, axis: Vec3::X },
                ));
            }
        )
        .id()
}

/// Core [`AircraftCoreBundle`] for the J-3 Cub root entity.
///
/// Mass, CoG, and inertia are computed by Avian from child zone colliders.
///
/// Pair with [`InducedDrag`] (already included by [`spawn`]) for lift-induced
/// drag.  No [`LodDamping`](avian_fdm::components::LodDamping). Roll/pitch/yaw
/// damping emerges from per-zone local α/β physics.
pub fn breeze_core_bundle(transform: Transform, initial_velocity: DVec3) -> impl Bundle {
    (AircraftCoreBundle {
        geometry: AircraftGeometry {
            wing_area_m2: WING_AREA_M2,
            wing_span_m: WING_SPAN_M,
            chord_m: CHORD_M,
        },
        rigid_body: RigidBody::Dynamic,
        linear_velocity: LinearVelocity(initial_velocity),
        transform,
        global_transform: GlobalTransform::default(),
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
    mass: Mass,
) -> impl Bundle {
    let ac_offset = DVec3::new(ac_x_m - x_m, 0.0, 0.0);
    let z_m = WING_Z - y_m.abs() * WING_DIHEDRAL_RAD.sin();
    let dihedral_rot = Quat::from_rotation_x(-(WING_DIHEDRAL_RAD * y_m.signum()) as f32);
    (
        Name::new(name),
        AeroZoneBundle {
            zone: AeroZone {
                cl: airfoil.cl,
                cd: airfoil.cd,
                ac_offset: ac_offset.as_vec3(),
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
        mass,
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
    let aileron_x = WING_AC_X - 2.8;
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
            transform: Transform::from_xyz(aileron_x as f32, y_m as f32, z_m as f32)
                .with_rotation(dihedral_rot),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// At alpha > 0 (nose up), the h-stab produces positive CL (upward force),
/// which at the aft arm creates a nose-down restoring moment.
///
/// The table is extended to +/-180 deg via Viterna-Corrigan post-stall model,
/// so the h-stab stalls realistically at high alpha and produces flat-plate
/// drag when broadside to the wind. This prevents unrealistic pitch-locking
/// during tumbles and deep stalls.
///
/// Elevator zone: pitch control surface.
///
/// Negative CL means: positive elevator (nose-up stick input) produces downward
/// force at the tail, creating a nose-up pitch moment via the tail arm.
pub fn elevator_zone(collider: Collider, density: ColliderDensity, y_m: Scalar) -> impl Bundle {
    let elevator_x = WING_AC_X - 2.8;
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
            transform: Transform::from_xyz(elevator_x as f32, y_m as f32, z_m as f32),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// Vertical tail zone: structural mass and weathercock stability.
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
            transform: Transform::from_xyz(-VFIN_ARM_M as f32, 0.0, -2.35),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

/// Rudder zone: yaw control surface.
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
            transform: Transform::from_xyz(
                -(VFIN_ARM_M + VFIN_MEAN_CHORD_M / 2.0 + RUDDER_MEAN_CHORD_M / 2.0) as f32,
                0.0,
                -2.0,
            ),
            global_transform: GlobalTransform::default(),
        },
        density,
    )
}

pub fn engine_zone(collider: Collider, mass: Mass, y_m: f64) -> impl Bundle {
    (
        EngineZone {
            max_thrust_n: sourced!(
                7_330.0 * 9.80665,
                "https://wiki.warthunder.com/unit/rafale_c_f3: Afterburner"
            ),
            throttle_curve: sourced!(
                vec![[0.0, 0.0], [0.5, 0.5], [0.9, 0.668_485_7], [1.0, 1.0]],
                "https://wiki.warthunder.com/unit/rafale_c_f3: Afterburner at 100%"
            ),
            thrust_axis_body: Vector::X, // +X = forward
            zero_thrust_speed_ms: Some(sourced!(400.0, "Estimate")),
        },
        collider,
        mass,
        Transform::from_xyz(-5.2, y_m as f32, -0.4),
        GlobalTransform::default(),
    )
}
