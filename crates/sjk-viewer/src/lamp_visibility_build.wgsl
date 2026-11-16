struct LampGrid { origin: vec4<i32>, counts: vec4<u32>, cell: vec4<f32>, offsets: vec4<u32>, selection:vec4<u32> };
@group(1) @binding(0) var<uniform> lamps: LampGrid;
@group(1) @binding(1) var<storage,read> data: array<vec4<u32>>;
// Two atlas texels per invocation (`lamp_visibility_pack`); tiles start at even texels.
@group(1) @binding(2) var distances: texture_storage_2d<r32uint,write>;
@compute @workgroup_size(8,8,1) fn build(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(distances);
    if any(id.xy>=size) { return; }
    let resolution = lamps.offsets.z;
    let pitch = resolution+2u;
    let even = vec2(id.x*2u,id.y);
    let tile = even/pitch;
    let lamp = tile.x+tile.y*lamps.offsets.w;
    // Mover door tiles (`mover_occlusion.wgsl`) start unblocked until traced: all misses,
    // which their first stored texel reads as no covered rectangle.
    if lamp>=lamps.counts.w {
        textureStore(distances,id.xy,vec4(select(0u,LAMP_DOOR_UNCOVERED,lamp<lamps.counts.w+lamps.selection.w)));
        return;
    }
    let step = lamp_visibility_step(bitcast<vec4<f32>>(data[lamp*5u]).w);
    let pair = lamp_visibility_pack(trace_texel(lamp,even,pitch),trace_texel(lamp,even+vec2(1u,0u),pitch),step);
    textureStore(distances,id.xy,vec4(pair));
}
// The biased ray parameter of the first blocker in the direction of atlas texel `at`.
fn trace_texel(lamp: u32, at: vec2<u32>, pitch: u32) -> f32 {
    let resolution = pitch-2u;
    let pixel = vec2<f32>(at%pitch)-1.0;
    let direction = lamp_octa_direction((pixel+0.5)/f32(resolution)*2.0-1.0);
    let source = bitcast<vec4<f32>>(data[lamp*5u]);
    let normal = bitcast<vec4<f32>>(data[lamp*5u+1u]).xyz;
    let hit = gi_trace_through(source.xyz+normal*0.5,direction,source.w+4.0,true);
    // A miss is not a blocker at the tracing limit. Filter rays grazing a
    // receiver can intersect its plane beyond that limit even inside lamp range.
    let ray_parameter = (hit.distance+0.5)*dot(abs(direction),vec3(1.0));
    return select(1e20,ray_parameter,hit.hit);
}
