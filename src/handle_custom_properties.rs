/*
This file handles importing gltfs from Blender with custom properties.

How to make buttons with custom properties (don't forget to export with custom properties enabled)
Colliders are automatically hidden.
Thanks to Christopher Biscardi for making a tutorial about it.
button: ButtonTypes
function: ButtonID
*/

use bevy::{gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady};
use serde::{Deserialize, Serialize};

// TODO a switch is more complex than a button, it needs some more fields
#[derive(Debug, Serialize, Deserialize)]
pub enum ButtonTypes {
    Switch,
    Button,
    Lever,
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub enum ButtonID {
    Button1,
    Button2,
    None,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Button {
    button: ButtonTypes,
    function: Option<ButtonID>,
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
        match data.button {
            ButtonTypes::Button => {
                let function;
                match data.function {
                    Some(ButtonID::Button1) => function = ButtonID::Button1,
                    Some(ButtonID::Button2) => function = ButtonID::Button2,
                    _ => function = ButtonID::None,
                }
                let bundle = (Pickable::default(), function);
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
}

#[derive(Debug, Deserialize)]
pub struct Light {
    light: Lights,
}

// TODO new observer for glass
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
        match data.light {
            Lights::AntiCol => {
                let material_emissive = materials.add(StandardMaterial {
                    emissive: LinearRgba::rgb(0.0, 0.0, 0.0),
                    ..default()
                });
                commands
                    .entity(entity)
                    .insert((MeshMaterial3d(material_emissive), Lights::AntiCol));
            }
            _ => {
                warn!("not handled yet")
            }
        }
    }
}
