/*
Sse stands for screen space effects.
It includes screen space reflections (and technically screen space ambient occlusion,
but it doesn't work for large terrain because of f32 accuracy)
*/

use bevy::{pbr::DefaultOpaqueRendererMethod, prelude::*};

pub struct Sse;
impl Plugin for Sse {
    fn build(&self, app: &mut App) {
        app.insert_resource(DefaultOpaqueRendererMethod::deferred());
    }
}
