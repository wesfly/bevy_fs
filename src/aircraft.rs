use std::time::Duration;

use crate::{
    Settings,
    data_from_gltf::{InterfaceOperation, InterfaceType, Lights},
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
    scene::SceneInstanceReady,
};

pub const STROBE_OFF_DURATION: f32 = 1.0;
pub const STROBE_ON_DURATION: f32 = 0.1;
pub const ACOL_OFF_DURATION: f32 = 1.2;
pub const ACOL_ON_DURATION: f32 = 0.1;

#[derive(Resource, Default)]
pub struct AircraftState {
    pub engine_on: bool,
    pub anti_col_lts_on: bool,
    pub pos_lts_on: bool,
    pub strobe_lts_on: bool,
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

pub fn update_lights(
    material_handles: Query<(&MeshMaterial3d<StandardMaterial>, &Lights, Entity), With<Lights>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    state: Res<AircraftState>,
    time: Res<Time>,
    mut timer: ResMut<LightsTimers>,
) {
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

#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}

pub fn spawn(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    settings: &Res<Settings>,
) {
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("aircraft.glb")),
    );
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // Aircraft collider
    let aircraft = commands
        .spawn((
            SceneRoot(asset_server.load("aircraft.glb#Scene1")),
            Aircraft,
            RigidBody::Dynamic,
            ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
            Transform::from_xyz(0., 20., 0.),
            Mass(5000.),
            Visibility::Hidden,
        ))
        .id();

    // The real aircraft model
    commands
        .spawn((
            SceneRoot(asset_server.load("aircraft.glb#Scene0")),
            Visibility::Visible,
            ChildOf(aircraft),
            animation_to_play,
            ColliderDisabled,
            RigidBodyDisabled,
        ))
        .observe(crate::buttons_from_gltf)
        .observe(crate::lights_from_gltf)
        .observe(play_animation_when_ready);

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
    if let Some(sse) = crate::sse_config(&settings) {
        camera.insert(sse);
    }

    if let Some(a) = motion_blur(&settings) {
        camera.insert(a);
    }
}

fn play_animation_when_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if let Ok(animation_to_play) = animations_to_play.get(scene_ready.entity) {
        for child in children.iter_descendants(scene_ready.entity) {
            if let Ok(mut player) = players.get_mut(child) {
                player.play(animation_to_play.index).repeat();
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()));
            }
        }
    }
}
