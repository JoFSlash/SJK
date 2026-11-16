// The Collection's blade pictures (`blade_pictures.rs`): each cell's blades, as the blade
// renderer drew them in display values, faded out softly toward the cell's edges, so a swatch
// never shows a straight cut where its picture stops (Creyon's round 13: "the start of the
// blades looks really bad"): the left over the blade's first units, where the hilt sits, the
// tip's end and the top and bottom over their last units.
const COLUMNS: f32 = 2.0;
const ROWS: f32 = 8.0;
// In shares of a cell (52 by 26 units): the blade starts 3 units in.
const LEFT: vec2<f32> = vec2<f32>(0.5 / 52.0, 4.5 / 52.0);
const RIGHT: f32 = 4.0 / 52.0;
const EDGE: f32 = 5.0 / 26.0;
@group(0) @binding(0) var blades: texture_2d<f32>;

@vertex fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 0.0, 1.0);
}

@fragment fn fs_main(@builtin(position) at: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(blades));
    let cell = fract(at.xy / size * vec2(COLUMNS, ROWS));
    let fade = smoothstep(LEFT.x, LEFT.y, cell.x)
        * (1.0 - smoothstep(1.0 - RIGHT, 1.0, cell.x))
        * smoothstep(0.0, EDGE, cell.y)
        * (1.0 - smoothstep(1.0 - EDGE, 1.0, cell.y));
    let c = textureLoad(blades, vec2<i32>(at.xy), 0).rgb * fade;
    return vec4(c, max(c.r, max(c.g, c.b)));
}

// A cell drawn alone (`blade_pictures.rs`), inside its scissor: cleared to nothing and its
// depth to the far plane, while the cells kept stay at the depth of 0 the pass clears to,
// in front of every blade (the blades test depth and never write it), so no glow spills
// into a picture kept from an earlier frame.
@vertex fn vs_clear(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let p = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(p * 2.0 - 1.0, 1.0, 1.0);
}

@fragment fn fs_clear() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0);
}
