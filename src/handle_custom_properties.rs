/*
This file handles importing gltfs from Blender with custom properties.

How to make colliders with custom properties (don't forget to export with CP enabled)
Colliders are automatically hidden.
Thanks to Christopher Biscardi for making a tutorial about it.
rigid_body: Static, Dynamic
collider: TrimeshFromMesh, Cuboid
*/

use avian3d::prelude::*;
use bevy::{gltf::GltfMeshExtras, prelude::*, scene::SceneInstanceReady};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BMeshExtras {
    pub collider: BCollider,
    pub rigid_body: BRigidBody,
    pub cube_size: Option<Vec3>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BCollider {
    TrimeshFromMesh,
    Cubiod,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BRigidBody {
    Static,
    Dynamic,
}

pub fn on_scene_spawn(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    extras: Query<&GltfMeshExtras>,
) {
    for entity in children.iter_descendants(trigger.entity.entity()) {
        let Ok(gltf_mesh_extras) = extras.get(entity) else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<BMeshExtras>(&gltf_mesh_extras.value) else {
            error!("couldn't deseralize extras");
            continue;
        };
        dbg!(&data);
        match data.collider {
            BCollider::TrimeshFromMesh => {
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => (RigidBody::Static, Visibility::Hidden),
                        BRigidBody::Dynamic => (RigidBody::Dynamic, Visibility::Hidden),
                    },
                    ColliderConstructor::TrimeshFromMesh,
                ));
            }
            BCollider::Cubiod => {
                let size = data.cube_size.expect("Cubiod collider must have cube_size");
                commands.entity(entity).insert((
                    match data.rigid_body {
                        BRigidBody::Static => (RigidBody::Static, Visibility::Hidden),
                        BRigidBody::Dynamic => (RigidBody::Dynamic, Visibility::Hidden),
                    },
                    Collider::cuboid(size.x, size.y, size.z),
                ));
            }
        }
    }
}
