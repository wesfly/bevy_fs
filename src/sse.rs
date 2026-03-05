/*
SSE stands for screen space effects.
It includes screen space reflections (and technically screen space ambient occlusion,
but it doesn't work for large terrain because of f32 accuracy)
*/

use crate::{Settings, scenery::water::Water};
use bevy::{
    anti_alias::fxaa::Fxaa,
    pbr::{DefaultOpaqueRendererMethod, ExtendedMaterial, ScreenSpaceReflections},
    prelude::*,
};

pub fn sse_config(settings: &Settings) -> Option<(ScreenSpaceReflections, Msaa, Fxaa)> {
    if settings.screen_space_effects {
        Some((
            ScreenSpaceReflections::default(),
            Msaa::Off,
            Fxaa::default(),
        ))
    } else {
        None
    }
}

pub fn insert_sse_resources(app: &mut App) {
    app.insert_resource(DefaultOpaqueRendererMethod::deferred())
        .add_plugins(MaterialPlugin::<ExtendedMaterial<StandardMaterial, Water>>::default());
}
