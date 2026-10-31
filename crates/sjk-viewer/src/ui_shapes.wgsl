struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) local: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) start_color: vec4<f32>,
    @location(4) end_color: vec4<f32>,
    @location(5) parameters: vec2<f32>,
    @location(6) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) start_color: vec4<f32>,
    @location(3) end_color: vec4<f32>,
    @location(4) parameters: vec2<f32>,
    @location(5) uv: vec2<f32>,
}

@group(0) @binding(0) var icon_texture: texture_2d<f32>;
@group(0) @binding(1) var icon_sampler: sampler;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4(input.position, 0.0, 1.0);
    output.local = input.local;
    output.size = input.size;
    output.start_color = input.start_color;
    output.end_color = input.end_color;
    output.parameters = input.parameters;
    output.uv = input.uv;
    return output;
}

// SJK's strafe aura, the same expressions as `sjk_ui::strafe_aura_shade` (aura.rs).
fn strafe_gain(speed: f32, acceleration: f32, wishspeed: f32, angle: f32) -> f32 {
    let along = speed * cos(angle);
    let add = clamp(wishspeed - along, 0.0, acceleration);
    let grow = 2.0 * along * add + add * add;
    let next = sqrt(max(speed * speed + grow, 0.0));
    if next + speed > 0.0 {
        return grow / (next + speed);
    }
    return 0.0;
}

// WGSL's pow is undefined for a negative base.
fn square(x: f32) -> f32 {
    return x * x;
}

fn wrap_angle(angle: f32) -> f32 {
    return angle - 6.2831855 * floor((angle + 3.1415927) / 6.2831855);
}

fn strafe_aura(input: VertexOutput) -> vec4<f32> {
    // start_color = (speed, acceleration, wish speed, base angle), end_color = (radius,
    // width, span, time), parameters.x = opacity, uv = the quad's corner from the centre.
    let speed = input.start_color.x;
    let acceleration = input.start_color.y;
    let wishspeed = input.start_color.z;
    let base = input.start_color.w;
    let radius = input.end_color.x;
    let half = input.end_color.y * 0.5;
    let span = input.end_color.z;
    let t = input.end_color.w;
    let pixel = input.uv + input.local * input.size;
    let across = (length(pixel) - radius) / half;
    if abs(across) > 1.6 || half <= 0.0 || speed <= 0.0 {
        discard;
    }
    let turn = wrap_angle(-1.5707964 - atan2(pixel.y, pixel.x));
    let fade = 1.0 - smoothstep(span - 0.15, span, abs(turn));
    let peak = acos(clamp((wishspeed - acceleration) / speed, -1.0, 1.0));
    let top = strafe_gain(speed, acceleration, wishspeed, peak);
    if fade <= 0.0 || top <= 0.0 {
        discard;
    }
    let angle = base + turn;
    let n = strafe_gain(speed, acceleration, wishspeed, angle) / top;
    let to_best = min(abs(wrap_angle(angle - peak)), abs(wrap_angle(angle + peak)));
    let flow = 0.5 + 0.5 * sin(to_best * 40.0 - t * 5.0 + sin(across * 3.0 + t * 1.7) * 1.2);
    let flicker = 0.5 + 0.5 * (sin(turn * 23.0 + t * 2.3 + across * 4.0) * sin(turn * 11.0 - t * 1.3));
    var pulse = 1.0;
    if strafe_gain(speed, acceleration, wishspeed, base) / top > 0.9 {
        pulse = 1.0 + 0.35 * sin(t * 9.0);
    }
    let seam = exp(-square(to_best / 0.02)) * exp(-square(across / 0.55));
    var rgb = vec3(1.0);
    var a = 0.0;
    if n > 0.0 {
        let hot = n * n;
        let flame = across / (0.35 + 0.65 * n) + (flow - 0.5) * 0.35;
        let body = exp(-flame * flame * 3.0);
        let core = exp(-square(across / 0.18));
        rgb = vec3(0.22 + 0.63 * hot, 0.5 + 0.45 * hot, 1.0);
        a = (body * (0.3 + 0.4 * flow + 0.2 * flicker) * sqrt(n) + core * 0.6 * hot) * pulse;
    } else if n < 0.0 {
        rgb = vec3(1.0, 0.28, 0.2);
        a = min(-n, 1.0) * 0.55 * exp(-square(across / 0.3)) * (0.7 + 0.3 * flow);
    }
    if seam > 0.0 {
        rgb = rgb + (vec3(1.0) - rgb) * seam;
        a = max(a, seam * 0.95 * min(pulse, 1.0));
    }
    a = clamp(a * fade * input.parameters.x, 0.0, 1.0);
    if a <= 0.0 {
        discard;
    }
    return vec4(rgb, a);
}

// A card's finish, the same expressions as `sjk_ui::card_shine_shade` (shine.rs).
// PCG hash of a lattice point, in [0, 1].
fn shine_hash(cell: vec2<i32>) -> f32 {
    var v = bitcast<u32>(cell.x) * 747796405u + 2891336453u;
    v = v ^ (bitcast<u32>(cell.y) * 2654435761u);
    v = v * 747796405u + 2891336453u;
    let word = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
    return f32((word >> 22u) ^ word) / 4294967295.0;
}

fn shine_noise(p: vec2<f32>) -> f32 {
    let c = floor(p);
    let f = p - c;
    let u = f * f * (3.0 - 2.0 * f);
    let cell = vec2<i32>(c);
    let a = shine_hash(cell);
    let b = shine_hash(cell + vec2(1, 0));
    let d = shine_hash(cell + vec2(0, 1));
    let e = shine_hash(cell + vec2(1, 1));
    let top = a + (b - a) * u.x;
    let bottom = d + (e - d) * u.x;
    return top + (bottom - top) * u.y;
}

// Straight-alpha `top` at `alpha` over straight-alpha `under`.
fn shine_over(under: vec4<f32>, top: vec3<f32>, alpha_in: f32) -> vec4<f32> {
    let alpha = clamp(alpha_in, 0.0, 1.0);
    let out = alpha + under.a * (1.0 - alpha);
    if out <= 0.0 {
        return vec4(0.0);
    }
    return vec4((top * alpha + under.rgb * under.a * (1.0 - alpha)) / out, out);
}

const SUN_CORE: vec3<f32> = vec3(1.0, 0.97, 0.86);
const SUN_INNER: vec3<f32> = vec3(1.0, 0.72, 0.24);
const SUN_RIM_HOT: vec3<f32> = vec3(1.0, 0.42, 0.05);
const SUN_RIM_COOL: vec3<f32> = vec3(1.0, 0.2, 0.02);
const SUN_MOTE: vec3<f32> = vec3(1.0, 0.82, 0.42);

fn shine_holo(p: vec2<f32>, size: vec2<f32>, t: f32, light: vec2<f32>, hold: f32) -> vec4<f32> {
    let w = size.x;
    let h = size.y;
    let u = p.x / w;
    let v = p.y / h;
    let across = (p.x - light.x * w) + 0.84 * (p.y - light.y * h);
    let k = across / (0.68 * w);
    let pearl = mix(vec3(0.75, 0.85, 1.0), vec3(1.0, 0.7, 0.95), u);
    var out = vec4(pearl, 0.05 + 0.03 * u);
    let hue = k * 0.9 + 0.5 + 0.07 * sin(u * 5.0 + v * 2.0 + t * 0.6);
    var rainbow = 0.5 + 0.5 * cos(6.2831855 * (vec3(hue) - vec3(0.0, 1.0 / 3.0, 2.0 / 3.0)));
    rainbow = rainbow + (vec3(1.0) - rainbow) * 0.3;
    let envelope = exp(-k * k * 2.2);
    let strength = 0.26 + 0.04 * clamp(hold, 0.0, 1.0);
    out = shine_over(out, rainbow, 0.035 + strength * envelope);
    let line = across / (0.03 * w);
    let halo = across / (0.12 * w);
    let glint = exp(-line * line) * 0.3 + exp(-halo * halo) * 0.07;
    return shine_over(out, vec3(1.0), glint);
}

fn shine_solar(p: vec2<f32>, inside: f32, size: vec2<f32>, t: f32, light: vec2<f32>) -> vec4<f32> {
    let w = size.x;
    let h = size.y;
    let reach = max(w * 0.06, 5.0);
    let u = p.x / w;
    let v = p.y / h;
    var out = vec4(SUN_INNER, 0.06 * (1.0 - v));
    let across = (p.x - light.x * w) + 0.7 * (p.y - light.y * h);
    let wide = across / (0.16 * w);
    let thin = across / (0.025 * w);
    out = shine_over(out, SUN_INNER, exp(-wide * wide) * 0.07);
    out = shine_over(out, SUN_CORE, exp(-thin * thin) * 0.13);
    if inside > reach * 3.2 {
        return out;
    }
    let angle = atan2(v - 0.5, u - 0.5);
    var flare = 0.0;
    for (var index = 0; index < 3; index++) {
        let i = f32(index);
        let direction = select(1.0, -1.0, index == 1);
        let centre = direction * t * (0.32 + 0.11 * i) + i * 2.1;
        let off = wrap_angle(angle - centre) / 0.3;
        let swell = 0.55 + 0.45 * sin(t * 1.3 + i * 1.7);
        flare += exp(-off * off) * swell;
    }
    let q = p / reach;
    var grain = 0.62 * shine_noise(vec2(q.x * 0.55 + t * 0.35, q.y * 0.55 - t * 0.9))
        + 0.38 * shine_noise(vec2(q.x * 1.3 - t * 0.6, q.y * 1.3 - t * 1.6 + 17.0));
    grain = smoothstep(0.15, 0.85, grain);
    let ridge = 1.0 - abs(2.0 * shine_noise(vec2(q.x * 0.8 + 31.0, q.y * 0.8 - t * 1.2)) - 1.0);
    let tongue = ridge * ridge * ridge;
    let height = reach * (0.6 + 0.7 * grain + 0.9 * tongue + 0.8 * flare);
    let depth = max(inside, 0.0) / height;
    let shimmer = 1.0 + 0.045 * sin(t * 9.0 + u * 4.4) + 0.03 * sin(t * 15.7 - v * 8.2);
    if depth < 1.0 {
        let body = pow(1.0 - depth, 1.6);
        let bright = min(0.45 + 0.4 * grain + 0.25 * tongue + 0.6 * flare, 1.0);
        let rim = mix(SUN_RIM_HOT, SUN_RIM_COOL, clamp(1.0 - grain, 0.0, 1.0));
        var colour = mix(SUN_INNER, rim, smoothstep(0.15, 0.85, depth));
        let c = max(inside, 0.0) / (reach * 0.16);
        let core = exp(-c * c) * (0.6 + 0.4 * min(flare, 1.0));
        colour = mix(colour, SUN_CORE, core);
        out = shine_over(out, colour, min(body * bright * shimmer, 0.95));
    }
    let cell = reach * 0.9;
    let rise = vec2(p.x / cell, (p.y + t * reach * 1.1) / cell);
    let base = floor(rise);
    let at = vec2<i32>(base);
    let seed = shine_hash(at);
    if seed > 0.86 {
        let mx = base.x + 0.2 + 0.6 * shine_hash(at + vec2(7, 0));
        let my = base.y + 0.2 + 0.6 * shine_hash(at + vec2(0, 7));
        let dx = (rise.x - mx) * cell;
        let dy = (rise.y - my) * cell * 0.6;
        let dot_size = max(reach * 0.09, 0.9);
        let spot = (dx * dx + dy * dy) / (dot_size * dot_size);
        let twinkle = 0.5 + 0.5 * sin(t * 3.0 + seed * 40.0);
        let fade = 1.0 - smoothstep(reach * 1.2, reach * 3.0, inside);
        out = shine_over(out, SUN_MOTE, exp(-spot) * twinkle * fade * 0.9);
    }
    return out;
}

fn shine_glare(p: vec2<f32>, size: vec2<f32>, light: vec2<f32>, hold: f32) -> vec4<f32> {
    let spread = 0.32 * min(size.x, size.y);
    let d = (p - light * size) / spread;
    return vec4(vec3(1.0), 0.16 * exp(-dot(d, d)) * clamp(hold, 0.0, 1.0));
}

fn card_shine(input: VertexOutput) -> vec4<f32> {
    // start_color = (kind, time, light across, light down): kind 0 Holo, 1 Solar, 2 the
    // glare; end_color = (tile width, height, corner radius, the pointer's hold);
    // parameters.x = opacity; uv = the quad's top-left from the tile's.
    let size = input.end_color.xy;
    let p = input.uv + input.local * input.size;
    let radius = clamp(input.end_color.z, 0.0, min(size.x, size.y) * 0.5);
    let q = abs(p - size * 0.5) - (size * 0.5 - vec2(radius));
    let outside = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
    let coverage = clamp(0.5 - outside, 0.0, 1.0);
    if coverage <= 0.0 || size.x <= 0.0 || size.y <= 0.0 {
        discard;
    }
    let kind = input.start_color.x;
    let hold = input.end_color.w;
    var out: vec4<f32>;
    if kind > 1.5 {
        out = shine_glare(p, size, input.start_color.zw, hold);
    } else if kind > 0.5 {
        out = shine_solar(p, -outside, size, input.start_color.y, input.start_color.zw);
    } else {
        out = shine_holo(p, size, input.start_color.y, input.start_color.zw, hold);
    }
    let a = clamp(out.a * coverage * input.parameters.x, 0.0, 1.0);
    if a <= 0.0 {
        discard;
    }
    return vec4(out.rgb, a);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    if input.parameters.y > 6.5 {
        // Convex polygon, a fan of triangles from its middle: local.x is the distance in
        // pixels to the triangle's outer edge, so coverage fades across the last pixel
        // inside the polygon. parameters.x is 1 for a textured one.
        let sampled = textureSample(icon_texture, icon_sampler, input.uv);
        let coverage = clamp(input.local.x, 0.0, 1.0);
        if coverage <= 0.0 {
            discard;
        }
        let color = select(input.start_color, sampled * input.start_color, input.parameters.x > 0.5);
        return vec4(color.rgb, color.a * coverage);
    }
    if input.parameters.y > 5.5 {
        return card_shine(input);
    }
    if input.parameters.y > 4.5 {
        return strafe_aura(input);
    }
    if input.parameters.y > 3.5 {
        // Straight stroke: the quad runs along the line with one pixel of margin on
        // each side; coverage fades across one pixel at the stroke's edges.
        // end_color.x = stroke width in pixels.
        let across = abs(input.local.y - 0.5) * input.size.y - input.end_color.x * 0.5;
        let coverage = clamp(0.5 - across, 0.0, 1.0);
        if coverage <= 0.0 {
            discard;
        }
        return vec4(input.start_color.rgb, input.start_color.a * coverage);
    }
    if input.parameters.y > 2.5 {
        // Arc stroke with round caps: distance to the nearest point of the centre
        // line, which is the circle's arc clamped to its angular range.
        // end_color = (radius, width, start angle, sweep) in pixels and radians. The
        // knockout stripe, where the stroke is left out, is parameters.x (left edge), uv.x
        // (right edge) and uv.y (half height), in pixels from the centre; a zero height
        // is no stripe.
        let radius = input.end_color.x;
        let half_width = input.end_color.y * 0.5;
        let sweep = input.end_color.w;
        let pixel = input.local * input.size - input.size * 0.5;
        let middle = input.end_color.z + sweep * 0.5;
        var delta = atan2(pixel.y, pixel.x) - middle;
        // Wrap to (-pi, pi] so the arc may cross the +/-pi seam.
        delta = delta - 6.2831855 * floor((delta + 3.1415927) / 6.2831855);
        let limit = abs(sweep) * 0.5;
        let angle = middle + clamp(delta, -limit, limit);
        let nearest = radius * vec2(cos(angle), sin(angle));
        let distance = length(pixel - nearest) - half_width;
        var coverage = clamp(0.5 - distance, 0.0, 1.0);
        if input.uv.y > 0.0 {
            // The same rounded-box distance as the panels, of an unrounded stripe
            // (`sjk_ui::knockout_coverage`). The panel there has this stroke's colour and
            // alpha, so only what it leaves is added (`sjk_ui::knockout_remainder`).
            let q = abs(vec2(pixel.x - (input.parameters.x + input.uv.x) * 0.5, pixel.y))
                - vec2((input.uv.x - input.parameters.x) * 0.5, input.uv.y);
            let outside = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0);
            let knock = clamp(0.5 - outside, 0.0, 1.0);
            coverage = clamp(
                (coverage - knock) / max(1.0 - input.start_color.a * knock, 1.0e-4),
                0.0,
                1.0,
            );
        }
        if coverage <= 0.0 {
            discard;
        }
        return vec4(input.start_color.rgb, input.start_color.a * coverage);
    }
    if input.parameters.y > 1.5 {
        return textureSample(icon_texture, icon_sampler, input.uv) * input.start_color;
    }
    let radius = input.parameters.x;
    // A negative parameters.y is a rounded border: its magnitude is the ring width.
    let ring = -input.parameters.y;
    var coverage = 1.0;
    if radius > 0.0 || ring > 0.0 {
        // Signed distance to the rounded outline in pixels; the edge is
        // anti-aliased by fading alpha across one pixel of it (and across
        // the inner edge of a ring) instead of a hard discard.
        let pixel = input.local * input.size;
        let half_size = input.size * 0.5;
        let q = abs(pixel - half_size) - (half_size - vec2(radius));
        let distance = length(max(q, vec2(0.0))) + min(max(q.x, q.y), 0.0) - radius;
        coverage = clamp(0.5 - distance, 0.0, 1.0);
        if ring > 0.0 {
            coverage *= clamp(distance + ring + 0.5, 0.0, 1.0);
        }
        if coverage <= 0.0 {
            discard;
        }
    }
    let amount = select(input.local.x, input.local.y, input.parameters.y > 0.5);
    let color = mix(input.start_color, input.end_color, amount);
    return vec4(color.rgb, color.a * coverage);
}
