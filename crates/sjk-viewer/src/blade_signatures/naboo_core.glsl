// Naboo Core: the glowing waters running through the core of Naboo, as a blade. Soft pulses of light
// swell slowly up it one after another, and two ribbons of light trail and sway beside it.
// @part Light pulses
// @part Light tendrils
// @room 8
// @palette {"corona.rim_cool":[0.14,0.1,0.48],"corona.rim_hot":[0.45,0.3,0.95],"corona.inner":[0.7,0.65,1.0],"core.white":[0.92,0.9,1.0],"core.fringe_cool":[0.4,0.3,0.95],"core.fringe_hot":[0.85,0.75,1.0],"light.color":[0.45,0.35,1.0],"trail":[0.55,0.42,1.0]}

vec3 naboo_core_glow0(vec3 g, vec2 b, float r, float px){
  float q = fract(b.y/12.0 - uTime*0.35);
  float bell = pow(q, 5.0)*(1.0 - smoothstep(0.9, 1.0, q));
  return g*(0.75 + 1.1*bell*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = naboo_core_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxStreams(ro, rd, px, y0, y1, 2.0, 1.8, 0.4, 2.0, 7.7, vec3(0.6, 0.85, 1.0))*0.7*part(1.0);
  return fx*uFx*2.40;
}
