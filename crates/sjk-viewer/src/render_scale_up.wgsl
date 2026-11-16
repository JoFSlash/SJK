// Render scale below 100 % (`render_scale.rs`): the scene, drawn smaller than the
// window, stretched over it with a bilinear filter, then a light contrast-adaptive
// sharpen against the filter's softness. The sharpen keeps RCAS's limits (AMD FidelityFX
// Super Resolution 1, MIT): the negative lobe of its four neighbours is the largest that
// neither pushes the pixel below its neighbours' darkest nor above their brightest
// channel, so edges get crisper without dark or bright halos.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var bilinear: sampler;
// 0 no sharpening, 1 the most RCAS allows.
override SHARPNESS: f32 = 0.5;

struct Stretched {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex fn vertex(@builtin(vertex_index) index: u32) -> Stretched {
    let positions = array(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    let p = positions[index];
    return Stretched(vec4(p, 0.0, 1.0), vec2(p.x * 0.5 + 0.5, 0.5 - p.y * 0.5));
}

// Linear scene colour, HDR values past 1 included, onto [0, 1) and back, so RCAS's
// limits (which assume a top of 1) hold: c / (1 + its largest channel), reversible.
fn compress(c: vec3<f32>) -> vec3<f32> {
    let positive = max(c, vec3(0.0));
    return positive / (1.0 + max(positive.r, max(positive.g, positive.b)));
}
fn expand(c: vec3<f32>) -> vec3<f32> {
    return c / max(1.0 - max(c.r, max(c.g, c.b)), 1e-4);
}
fn tap(uv: vec2<f32>) -> vec3<f32> {
    return compress(textureSampleLevel(source, bilinear, uv, 0.0).rgb);
}

@fragment fn fragment(in: Stretched) -> @location(0) vec4<f32> {
    let centre = textureSampleLevel(source, bilinear, in.uv, 0.0);
    if SHARPNESS <= 0.0 {
        return centre;
    }
    // Neighbours one scene texel away: the bilinear stretch blurs over one texel.
    let texel = 1.0 / vec2<f32>(textureDimensions(source));
    let e = compress(centre.rgb);
    let b = tap(in.uv - vec2(0.0, texel.y));
    let d = tap(in.uv - vec2(texel.x, 0.0));
    let f = tap(in.uv + vec2(texel.x, 0.0));
    let h = tap(in.uv + vec2(0.0, texel.y));
    let lowest = min(min(b, d), min(f, h));
    let highest = max(max(b, d), max(f, h));
    let hit_low = min(lowest, e) / max(4.0 * highest, vec3(1e-5));
    let hit_high = (1.0 - max(highest, e)) / (4.0 * lowest - 4.0);
    let lobes = max(-hit_low, hit_high);
    let lobe = max(-0.1875, min(max(lobes.r, max(lobes.g, lobes.b)), 0.0)) * SHARPNESS;
    let sharpened = (lobe * (b + d + f + h) + e) / (4.0 * lobe + 1.0);
    return vec4(expand(clamp(sharpened, vec3(0.0), vec3(0.9999))), centre.a);
}
