// A menu picture (`menu_pictures_paint.rs`) as the first thing in the world pass: the
// picture's display values turned back into the scene colour the final resolve shows
// as them (as `effect_layer.wgsl`'s `scene_value` does), crossfading from the last
// picture to the next. Concatenated with `post_hdr.wgsl` for the exposure.
//
// ENCODING: 0 the scene stores display values already (UNORM target), 1 it is an sRGB
// target sampled as linear, 2 it is a floating HDR scene shown through `hdr_encoded`.
override ENCODING: u32 = 1u;

struct Controls {
    // The share of the picture the screen shows, each way (cover fit).
    scale: vec2<f32>,
    // 0 shows `previous` alone, 1 `current` alone.
    mix: f32,
    _pad: f32,
}
@group(0) @binding(0) var current: texture_2d<f32>;
@group(0) @binding(1) var previous: texture_2d<f32>;
@group(0) @binding(2) var picture_sampler: sampler;
@group(0) @binding(3) var<uniform> controls: Controls;

struct Out {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex fn vs_main(@builtin(vertex_index) index: u32) -> Out {
    let p = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u)) * 2.0 - 1.0;
    var out: Out;
    // On the far plane, so whatever the world pass draws next stays in front.
    out.position = vec4<f32>(p, 1.0, 1.0);
    out.uv = vec2<f32>(0.5 + 0.5 * p.x * controls.scale.x, 0.5 - 0.5 * p.y * controls.scale.y);
    return out;
}

// `effect_layer.wgsl`'s inverse of `hdr_map`'s shoulder.
fn hdr_unmap(m: vec3<f32>) -> vec3<f32> {
    let peak = max(m.r, max(m.g, m.b));
    if peak <= 0.9 { return m; }
    let k = min(peak, 0.999) - 0.9;
    return m * ((0.9 + 0.1 * k / (0.1 - k)) / peak);
}

// Scene colour the final resolve shows as display value `g`.
fn scene_value(g: vec3<f32>) -> vec3<f32> {
    if ENCODING == 0u { return ldr_unexposed(g); }
    if ENCODING == 1u { return ldr_unexpose(srgb_decode(g), scene_exposure.exposure); }
    return hdr_unmap(srgb_decode(g)) / scene_exposure.exposure;
}

@fragment fn fs_main(in: Out) -> @location(0) vec4<f32> {
    let next = textureSampleLevel(current, picture_sampler, in.uv, 0.0).rgb;
    let last = textureSampleLevel(previous, picture_sampler, in.uv, 0.0).rgb;
    let g = mix(last, next, controls.mix);
    return vec4<f32>(scene_value(g), 1.0);
}
