/*
This file handles importing gltfs from Blender with custom properties.

How to make buttons with custom properties (don't forget to export with custom properties enabled)
Thanks to Christopher Biscardi for making a tutorial about it.

load
=================
Buttons:
interface_type: InterfaceType
operation: InterfaceOperation
inverse: bool
-----------------
Lights:
light: Lights
-----------------
Rotors:
rotor: RotorTypes
-----------------
Shading:
not_shadow_caster: bool
*/

use bevy::{gltf::GltfMeshExtras, light::NotShadowCaster, prelude::*, scene::SceneInstanceReady};
use serde::{Deserialize, Serialize};

#[derive(Debug, Component, Serialize, Deserialize)]
pub enum InterfaceType {
    Switch,
    Button,
    Lever,
}

#[derive(Component, Debug, Serialize, Deserialize)]
pub enum InterfaceOperation {
    AntiColLt,
    Engine,
    PositionLt,
    StrobeLt,
}

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Button {
    pub interface_type: InterfaceType,
    pub operation: Option<InterfaceOperation>,
    pub inverse: Option<bool>,
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

#[derive(Debug, Deserialize, Component)]
pub enum RotorTypes {
    Main,
    Rear,
}
#[derive(Debug, Deserialize)]
pub struct Rotor {
    rotor: RotorTypes,
}

#[derive(Debug, Deserialize)]
struct ShadingFromGltf {
    not_shadow_caster: bool,
}

pub fn load(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    mesh_extras: Query<&GltfMeshExtras>,
    other_extras: Query<&GltfExtras>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in children.iter_descendants(trigger.entity.entity()) {
        if let Ok(gltf_mesh_extras) = mesh_extras.get(entity) {
            if let Ok(light_mesh_data) = serde_json::from_str::<Light>(&gltf_mesh_extras.value) {
                dbg!(&light_mesh_data);
                let material_emissive = materials.add(StandardMaterial {
                    perceptual_roughness: 0.1,
                    specular_transmission: 1.0,
                    base_color: Color::LinearRgba(LinearRgba::rgb(0.5, 0.5, 0.5).with_alpha(0.5)),
                    ..default()
                });
                commands
                    .entity(entity)
                    .insert((MeshMaterial3d(material_emissive), light_mesh_data.light));
            };

            if let Ok(rotor_data) = serde_json::from_str::<Rotor>(&gltf_mesh_extras.value) {
                dbg!(&rotor_data);
                commands.entity(entity).insert(rotor_data.rotor);
            };

            if let Ok(shading_data) =
                serde_json::from_str::<ShadingFromGltf>(&gltf_mesh_extras.value)
            {
                if shading_data.not_shadow_caster {
                    dbg!(&shading_data);
                    commands.entity(entity).insert(NotShadowCaster);
                }
            };

            if let Ok(button_data) = serde_json::from_str::<Button>(&gltf_mesh_extras.value) {
                dbg!(&button_data);
                match button_data.interface_type {
                    InterfaceType::Button | InterfaceType::Switch => {
                        let bundle = (
                            Pickable::default(),
                            Button {
                                interface_type: button_data.interface_type,
                                inverse: button_data.inverse,
                                operation: button_data.operation,
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
            };
        };

        if let Ok(gltf_other_extras) = other_extras.get(entity) {
            if let Ok(light_data) = serde_json::from_str::<Light>(&gltf_other_extras.value) {
                dbg!(&light_data);
                commands.entity(entity).insert(light_data.light);
            };
        };
    }
}
