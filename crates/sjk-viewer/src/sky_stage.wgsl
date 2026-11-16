// Camera-relative Q3 outer sky box and sky-surface depth mask.
struct Camera {
    view_projection: mat4x4<f32>,
    camera_position: vec3<f32>,
    shader_time: f32,
};

// What one unit of the sky's image is in the scene's light units: one for a scene shown as it
// is lit, less for a scene whose final pass exposes it (the day program applies it).
override sky_radiance: f32 = 1.0;
@group(0) @binding(0) var<uniform> camera: Camera;
@group(1) @binding(0) var sky_images: texture_2d_array<f32>;
@group(1) @binding(1) var sky_sampler: sampler;
@group(1) @binding(2) var cloud_images: texture_2d_array<f32>;
@group(1) @binding(3) var cloud_sampler: sampler;
@group(1) @binding(4) var<uniform> clouds: Clouds;

// The shader's own stages over the box, rd-vanilla's cloud layer (`sky_clouds.rs`).
struct CloudStage {
    tcmod_a: array<vec4<f32>, 4>,
    tcmod_b: array<vec4<f32>, 4>,
    control: vec4<f32>,
    colour: vec4<f32>,
    rgb_wave: vec4<f32>,
    alpha_wave: vec4<f32>,
    generators: vec4<f32>,
    test: vec4<f32>,
};
struct Clouds {
    header: vec4<f32>,
    stages: array<CloudStage, 8>,
};

struct BoxInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) layer: f32,
};

struct BoxOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) layer: u32,
    // Camera-relative box position: the view direction of the texel.
    @location(2) direction: vec3<f32>,
};

@vertex
fn box_vertex(input: BoxInput) -> BoxOutput {
    var output: BoxOutput;
    let world = camera.camera_position + input.position;
    var clip = camera.view_projection * vec4<f32>(world, 1.0);
    clip.z = clip.w;
    output.position = clip;
    output.uv = input.uv;
    output.layer = u32(input.layer);
    output.direction = input.position;
    return output;
}

@fragment
fn box_fragment(input: BoxOutput) -> @location(0) vec4<f32> {
    let texel = with_clouds(textureSample(sky_images, sky_sampler, input.uv, input.layer),
        input.direction);
    return texel;
}

// rd-vanilla's wave table (`tr_init.cpp` R_InitFuncTables), noise and random as zero.
fn wave_value(function: f32, wave: vec4<f32>) -> f32 {
    let x = fract(wave.z + camera.shader_time * wave.w);
    var table = sin(x * 6.28318530718);
    let code = i32(function);
    if code == 1 {
        table = select(-1.0, 1.0, x < 0.5);
    } else if code == 2 {
        table = select(select(4.0 * x - 4.0, 2.0 - 4.0 * x, x < 0.75), 4.0 * x, x < 0.25);
    } else if code == 3 {
        table = x;
    } else if code == 4 {
        table = 1.0 - x;
    } else if code >= 5 {
        table = 0.0;
    }
    return wave.x + wave.y * table;
}

// `R_InitSkyTexCoords`: where the view ray meets a dome `height` above a world of radius
// 4096 centred under the eye, as the arc cosines of its unit vector's x and y.
fn cloud_coordinates(direction: vec3<f32>, height: f32) -> vec2<f32> {
    let radius = 4096.0;
    let d = normalize(direction);
    let p = -radius * d.z + sqrt(radius * radius * d.z * d.z + 2.0 * radius * height + height * height);
    let v = normalize(d * p + vec3(0.0, 0.0, radius));
    return vec2(acos(clamp(v.x, -1.0, 1.0)), acos(clamp(v.y, -1.0, 1.0)));
}

// The stage's tcMods in order, as the world's stages apply them (`stage_runtime.wgsl`).
fn cloud_tcmods(stage: CloudStage, initial: vec2<f32>, position: vec3<f32>) -> vec2<f32> {
    var uv = initial;
    let time = camera.shader_time;
    for (var index = 0; index < 4; index++) {
        if index >= i32(stage.control.y) { break; }
        let a = stage.tcmod_a[index];
        let b = stage.tcmod_b[index];
        let kind = i32(a.x);
        if kind == 1 {
            uv += fract(vec2(a.y, a.z) * time);
        } else if kind == 2 {
            uv *= vec2(a.y, a.z);
        } else if kind == 3 {
            let angle = -a.y * time * 6.28318530718 / 360.0;
            let centered = uv - vec2(0.5);
            uv = vec2(cos(angle) * centered.x - sin(angle) * centered.y,
                      sin(angle) * centered.x + cos(angle) * centered.y) + vec2(0.5);
        } else if kind == 4 {
            let now = a.w + time * b.x;
            uv.x += sin(((position.x + position.z) / 1024.0 + now) * 6.28318530718) * a.z;
            uv.y += sin((position.y / 1024.0 + now) * 6.28318530718) * a.z;
        } else if kind == 5 {
            let value = 1.0 / wave_value(a.y, vec4(a.z, a.w, b.x, b.y));
            uv = uv * value + vec2(0.5 - 0.5 * value);
        } else if kind == 6 {
            uv = vec2(uv.x * a.y + uv.y * a.w + b.y, uv.x * a.z + uv.y * b.x + b.z);
        }
    }
    return uv;
}

fn blend_factor(code: f32, source: vec4<f32>, destination: vec4<f32>) -> vec4<f32> {
    switch i32(code) {
        case 0: { return vec4(0.0); }
        case 2: { return source; }
        case 3: { return vec4(1.0) - source; }
        case 4: { return vec4(source.a); }
        case 5: { return vec4(1.0 - source.a); }
        case 6: { return destination; }
        case 7: { return vec4(1.0) - destination; }
        case 8: { return vec4(destination.a); }
        case 9: { return vec4(1.0 - destination.a); }
        case 10: {
            let f = min(source.a, 1.0 - destination.a);
            return vec4(f, f, f, 1.0);
        }
        default: { return vec4(1.0); }
    }
}

// The box texel seen along `direction` with the cloud stages blended over it, each as
// its blendFunc would over the framebuffer. Like `FillCloudBox`, nothing below the box's
// bottom face; whatever a stage covers counts as sky drawn.
fn with_clouds(box_texel: vec4<f32>, direction: vec3<f32>) -> vec4<f32> {
    let count = i32(clouds.header.x);
    if count == 0 { return box_texel; }
    let a = abs(direction);
    let below = direction.z < 0.0 && a.z >= a.x && a.z >= a.y;
    let base = cloud_coordinates(direction, clouds.header.y);
    let position = camera.camera_position + normalize(direction) * 1024.0;
    var colour = box_texel;
    var drawn = box_texel.a;
    for (var index = 0; index < 8; index++) {
        if index >= count { break; }
        let stage = clouds.stages[index];
        var uv = cloud_tcmods(stage, base, position);
        if stage.test.y > 0.0 { uv = clamp(uv, vec2(0.0), vec2(1.0)); }
        var source = textureSample(cloud_images, cloud_sampler, uv, i32(stage.control.x));
        var tint = stage.colour;
        if stage.generators.x > 0.0 { tint = vec4(vec3(clamp(wave_value(stage.generators.z, stage.rgb_wave), 0.0, 1.0)), tint.a); }
        if stage.generators.y > 0.0 { tint.a = clamp(wave_value(stage.generators.w, stage.alpha_wave), 0.0, 1.0); }
        source *= tint;
        let test = i32(stage.test.x);
        let passes = test == 0 || (test == 1 && source.a > 0.0) || (test == 2 && source.a < 0.5)
            || (test == 3 && source.a >= 0.5) || (test == 4 && source.a >= 0.75);
        let blended = source * blend_factor(stage.control.z, source, colour)
            + colour * blend_factor(stage.control.w, source, colour);
        let applies = passes && !below;
        colour = select(colour, clamp(blended, vec4(0.0), vec4(1.0)), applies);
        drawn = select(drawn, 1.0, applies);
    }
    return vec4(colour.rgb, drawn);
}

// A sky face's depth, pushed back by `SKY_FACE_PUSH` of its distance (the clip-space
// share between its depth and the far plane's). The world's absolute coordinates round
// the depth of a far face by some tenths of a percent of its distance, so a ceiling a
// unit or two under the sky brush it is built against fought with the sky (and the
// clouds over it) and flickered as the camera moved (jofmemorial, Sol's note of
// 12/10/2026). rd-vanilla draws its box at the far plane and never covers the world;
// pushed this little, the sky still covers whatever lies further behind its faces.
const SKY_FACE_PUSH: f32 = 1.0 / 32.0;

fn sky_face_clip(clip: vec4<f32>, push: f32) -> vec4<f32> {
    return vec4(clip.xy, clip.z + push * (clip.w - clip.z), clip.w);
}

struct MaskInput {
    @location(0) position: vec3<f32>,
};

@vertex
fn mask_vertex(input: MaskInput) -> @builtin(position) vec4<f32> {
    return sky_face_clip(camera.view_projection * vec4<f32>(input.position, 1.0),
        SKY_FACE_PUSH);
}

@fragment
fn mask_fragment() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}

// The box on the sky faces themselves (`draw_faces_shaded`): the texel the box would show
// along the fragment's view direction. Axis and (s, t) invert `make_sky_vec`.
struct FaceOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) direction: vec3<f32>,
};

@vertex
fn face_vertex(input: MaskInput) -> FaceOutput {
    return FaceOutput(sky_face_clip(camera.view_projection * vec4<f32>(input.position, 1.0),
        SKY_FACE_PUSH), input.position - camera.camera_position);
}

// (u, v, layer) of the box texel seen along `d`.
fn box_coordinates(d: vec3<f32>) -> vec3<f32> {
    let a = abs(d);
    var st = vec2(0.0);
    var layer = 0.0;
    if a.x >= a.y && a.x >= a.z {
        if d.x > 0.0 { st = vec2(-d.y, d.z)/a.x; layer = 0.0; } else { st = vec2(d.y, d.z)/a.x; layer = 1.0; }
    } else if a.y >= a.z {
        if d.y > 0.0 { st = vec2(d.x, d.z)/a.y; layer = 2.0; } else { st = vec2(-d.x, d.z)/a.y; layer = 3.0; }
    } else {
        if d.z > 0.0 { st = vec2(-d.y, -d.x)/a.z; layer = 4.0; } else { st = vec2(-d.y, d.x)/a.z; layer = 5.0; }
    }
    return vec3((st.x + 1.0)*0.5, 1.0 - (st.y + 1.0)*0.5, layer);
}
// BOX_COORDINATES_END
fn box_texel(d: vec3<f32>) -> vec4<f32> {
    let at = box_coordinates(d);
    // The box magnifies its faces; an explicit level also keeps the axis seams clean.
    return textureSampleLevel(sky_images, sky_sampler, at.xy, i32(at.z), 0.0);
}

@fragment
fn face_fragment(input: FaceOutput) -> @location(0) vec4<f32> {
    let texel = with_clouds(box_texel(input.direction), input.direction);
    // A missing box face without clouds has no geometry: whatever was there stays.
    if texel.a <= 0.0 { discard; }
    return texel;
}
