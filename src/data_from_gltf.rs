/*
This file handles importing gltfs from Blender with custom properties.

How to make buttons with custom properties (don't forget to export with custom properties enabled)
Thanks to Christopher Biscardi for making a tutorial about it.

buttons_from_gltf
-----------------
button: ButtonTypes
function: ButtonID
inverse: bool

lights_from_gltf
----------------
light: Lights
*/

use bevy::{gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady};
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Serialize, Deserialize)]
pub enum InputDeviceType {
    Switch,
    Button,
    Lever,
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub enum Function {
    AntiColLt,
    Engine,
    PositionLt,
    StrobeLt,
}

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Button {
    pub input_device_type: InputDeviceType,
    pub function: Option<Function>,
    pub inverse: Option<bool>,
}

pub fn buttons_from_gltf(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    for entity in children.iter_descendants(trigger.entity.entity()) {
        let Ok(gltf_mesh_extras) = extras.get(entity) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<Button>(&gltf_mesh_extras.value) else {
            continue;
        };

        #[cfg(debug_assertions)]
        dbg!(&data);
        match data.input_device_type {
            InputDeviceType::Button | InputDeviceType::Switch => {
                let bundle = (
                    Pickable::default(),
                    Button {
                        input_device_type: data.input_device_type,
                        inverse: data.inverse,
                        function: data.function,
                    },
                );
                commands
                    .entity(entity)
                    .insert(bundle)
                    .observe(crate::aircraft::button_listener);
            }
            _ => {
                warn!("not handled yet")
            }
        }
    }
}

#[derive(Deserialize, Debug, Component)]
pub enum Lights {
    AntiCol,
    Strobe,
    PositionPort,
    PositionStarboard,
    PositionRear,
}

#[derive(Debug, Deserialize)]
pub struct Light {
    light: Lights,
}

// TODO new observer for glass (needs proper shadow transparency)
// TODO move stick with input
pub fn lights_from_gltf(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in children.iter_descendants(trigger.entity.entity()) {
        let Ok(gltf_mesh_extras) = extras.get(entity) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<Light>(&gltf_mesh_extras.value) else {
            continue;
        };
        let material_emissive = materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.0, 0.0, 0.0),
            ..default()
        });

        commands
            .entity(entity)
            .insert((MeshMaterial3d(material_emissive), data.light));
    }
}
