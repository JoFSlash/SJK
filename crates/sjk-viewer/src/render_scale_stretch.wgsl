// A scene drawn smaller than the window (`r_renderScale`), stretched to it before
// postprocessing and the HUD. The coordinates come from the full-screen triangle, so the
// shader takes no window size.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
struct Stretch {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}
@vertex fn vertex(@builtin(vertex_index) index: u32) -> Stretch {
    let positions = array(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    let position = positions[index];
    return Stretch(vec4(position, 0.0, 1.0), vec2(position.x * 0.5 + 0.5, 0.5 - position.y * 0.5));
}
@fragment fn fragment(input: Stretch) -> @location(0) vec4<f32> {
    return textureSampleLevel(source, linear, input.uv, 0.0);
}
