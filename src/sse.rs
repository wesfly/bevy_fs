/*
SSE stands for screen space effects.
It includes screen space reflections (and technically screen space ambient occlusion,
but it doesn't work for large terrain because of f32 accuracy)
*/

use crate::scenery::water::Water;
use bevy::{
    pbr::{DefaultOpaqueRendererMethod, ExtendedMaterial},
    prelude::*,
};

pub struct SSE;
impl Plugin for SSE {
    fn build(&self, app: &mut App) {
        app.insert_resource(DefaultOpaqueRendererMethod::deferred())
            .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());
    }
}
