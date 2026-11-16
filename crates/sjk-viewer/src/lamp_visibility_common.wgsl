// Shared octahedral mapping for map-lifetime lamp visibility and its receiver.
// Unnormalized octahedral rays have unit L1 length. The static atlas stores
// biased ray parameters in this basis, avoiding normalization in each PCF tap.
fn lamp_octa_vector(p: vec2<f32>) -> vec3<f32> {
    var n = vec3(p, 1.0-abs(p.x)-abs(p.y));
    if n.z < 0.0 {
        n = vec3((1.0-abs(n.yx))*select(vec2(-1.0),vec2(1.0),n.xy>=vec2(0.0)),n.z);
    }
    return n;
}
fn lamp_octa_direction(p: vec2<f32>) -> vec3<f32> {
    return normalize(lamp_octa_vector(p));
}
fn lamp_octa_coordinates(direction: vec3<f32>) -> vec2<f32> {
    let n = direction/(abs(direction.x)+abs(direction.y)+abs(direction.z));
    if n.z < 0.0 {
        return (1.0-abs(n.yx))*select(vec2(-1.0),vec2(1.0),n.xy>=vec2(0.0));
    }
    return n.xy;
}
// The atlas keeps two horizontally neighbouring texels in one R32Uint texel, 16 bits each,
// the even texel in the low half (`lamp_visibility.rs`): a ray parameter in steps of
// 1/65534 of the longest a hit can have (the lamp's tracing limit, its radius and four
// units, with the half-unit bias, along a diagonal of L1 length √3), 0.03 units for a
// lamp of 1000. It is rounded to the nearest step, so blockers move neither way on
// average, but never stored more than a quarter unit nearer than traced (only a lamp of
// 37,000 units has steps that coarse): rounding never eats more than half the half-unit
// bias that keeps a receiver from shadowing itself. 65535 is a miss.
const LAMP_VISIBILITY_MISS: u32 = 0xffffu;
fn lamp_visibility_step(radius: f32) -> f32 {
    return (radius+4.5)*(1.7320508/65534.0);
}
fn lamp_visibility_code(parameter: f32, step: f32) -> u32 {
    if parameter >= 1e19 { return LAMP_VISIBILITY_MISS; }
    var code = round(parameter/step);
    if code*step < parameter-0.25 { code += 1.0; }
    return u32(min(code, 65534.0));
}
fn lamp_visibility_pack(even: f32, odd: f32, step: f32) -> u32 {
    return lamp_visibility_code(even, step) | (lamp_visibility_code(odd, step) << 16u);
}
// A door tile's first stored texel (two border texels) when no mover is in view, or
// before its first trace, when it holds two misses (`mover_occlusion.wgsl`); otherwise
// the covered pixel rectangle, a byte per bound.
const LAMP_DOOR_UNCOVERED: u32 = 0xffffffffu;
