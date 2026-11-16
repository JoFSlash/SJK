// Ryloth Dawn: sunrise over the mountains of Ryloth, as a blade. Warm orange and gold bands of
// light glow up the blade, soft as a sunrise, with a faint warm glow round it.
// @part Dawn glow
// @haze Warm haze
// @room 8
// @palette {"corona.rim_cool":[0.6,0.25,0.02],"corona.rim_hot":[1.0,0.62,0.12],"corona.inner":[1.0,0.85,0.45],"core.white":[1.0,0.95,0.8],"core.fringe_cool":[1.0,0.55,0.08],"core.fringe_hot":[1.0,0.78,0.35],"light.color":[1.0,0.6,0.1],"trail":[1.0,0.6,0.12]}

vec3 ryloth_dawn_glow0(vec3 g, vec2 b, float r, float px){
  float band = 0.5 + 0.5*sin(b.y*0.35 - uTime*0.25);
  return recol(g, mix(vec3(1.0, 0.5, 0.15), vec3(1.0, 0.82, 0.4), band));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = ryloth_dawn_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxAura(ro, rd, y0, y1, 2.0, vec3(1.0, 0.6, 0.25))*0.25*part(0.0);
  return fx*uFx*2.00;
}
