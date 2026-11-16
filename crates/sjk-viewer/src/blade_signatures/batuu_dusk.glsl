// Batuu Dusk: evening over the spires of Batuu, as a blade. Warm orange near the hilt fading into
// deep violet at the top, and the first stars appearing round its upper half.
// @part Evening stars
// @wisps Dusk haze
// @room 8
// @palette {"corona.rim_cool":[0.55,0.18,0.55],"corona.rim_hot":[1.0,0.45,0.3],"corona.inner":[1.0,0.8,0.7],"core.white":[1.0,0.93,0.88],"core.fringe_cool":[0.9,0.4,0.6],"core.fringe_hot":[1.0,0.75,0.6],"light.color":[0.9,0.42,0.5],"trail":[0.85,0.38,0.62]}

vec3 batuu_dusk_glow0(vec3 g, vec2 b, float r, float px){
  float m = smoothstep(0.1, 0.9, b.y/uLength);
  return recol(g, mix(vec3(1.0, 0.55, 0.2), vec3(0.6, 0.35, 1.0), m));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = batuu_dusk_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, L*0.55, y1, 0.8, 3.0, 8.0, 0.008, 3.5, vec3(0.9, 0.9, 1.0))*0.6*part(0.0);
  return fx*uFx*2.00;
}
