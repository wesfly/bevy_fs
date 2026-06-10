// The terrain material

#import bevy_pbr::{
    pbr_deferred_functions::deferred_output,
    pbr_fragment::pbr_input_from_standard_material,
    prepass_io::{VertexOutput, FragmentOutput},
}
#import bevy_render::globals::Globals

@group(0) @binding(1) var<uniform> globals: Globals;

@group(#{MATERIAL_BIND_GROUP}) @binding(103) var normals_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(104) var normals_sampler: sampler;

// Samples a single octave of noise and returns the resulting normal.
fn sample_noise_octave(uv: vec2<f32>, strength: f32) -> vec3<f32> {
    let N = textureSample(normals_texture, normals_sampler, uv).rbg * 2.0 - 1.0;
    // This isn't slerp, but it's good enough.
    return normalize(mix(vec3(0.0, 1.0, 0.0), N, strength));
}

// Samples all four octaves of noise and returns the resulting normal.
fn sample_noise(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let uv0 = uv * 100.0 + 123.0;
    let uv1 = uv * 200.0 + 234.0;
    let uv2 = uv * 300.0 + 345.0;
    let uv3 = uv * 400.0 + 456.0;
    return normalize(
        sample_noise_octave(uv0, 0.2) +
        sample_noise_octave(uv1, 0.2) +
        sample_noise_octave(uv2, 0.2) +
        sample_noise_octave(uv3, 0.2)
    );
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    // Create the PBR input.
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    // Bump the normal.
    pbr_input.N += sample_noise(in.uv, globals.time);

    let slope = length(cross(pbr_input.N, vec3(0.0, 1.0, 0.0)));


    // Based on mediterranean terrain
    if slope > 0.6 {
        pbr_input.material.base_color.r = 0.55;
        pbr_input.material.base_color.g = 0.5;
        pbr_input.material.base_color.b = 0.33;
    } else if slope > 0.3 {
        pbr_input.material.base_color.r += 0.08;
        pbr_input.material.base_color.g += 0.15;
        pbr_input.material.base_color.b += 0.12;
    } else {
        pbr_input.material.base_color.r += 0.25 - slope * 0.4;
        pbr_input.material.base_color.g += 0.3 - slope * 0.1;
        pbr_input.material.base_color.b += 0.2 - slope * 0.4;
    }


    // Send the rest to the deferred shader.
    return deferred_output(in, pbr_input);
}
