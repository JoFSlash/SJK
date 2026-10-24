// A scene drawn smaller than the window (`r_renderScale`), stretched to it before
// postprocessing and the HUD.
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var linear: sampler;
override OUTPUT_WIDTH: f32 = 1.0;
override OUTPUT_HEIGHT: f32 = 1.0;
@vertex fn vertex(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array(vec2(-1.0,-1.0), vec2(3.0,-1.0), vec2(-1.0,3.0));
    return vec4(positions[index], 0.0, 1.0);
}
@fragment fn fragment(@builtin(position) pixel: vec4<f32>) -> @location(0) vec4<f32> {
    return textureSampleLevel(source, linear, pixel.xy / vec2(OUTPUT_WIDTH, OUTPUT_HEIGHT), 0.0);
}
