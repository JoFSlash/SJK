// Smoked Transparisteel: a pane of smoked transparisteel, as a blade. A dark tinted blade with its
// core light softly showing through, and thin dark smoke curling off it.
// @part Smoke
// @room 8
// @palette {"corona.rim_cool":[0.25,0.2,0.18],"corona.rim_hot":[0.75,0.62,0.5],"corona.inner":[1.0,0.88,0.72],"core.white":[1.0,0.92,0.8],"core.fringe_cool":[0.6,0.45,0.35],"core.fringe_hot":[1.0,0.82,0.65],"light.color":[0.85,0.65,0.45],"trail":[0.6,0.5,0.42]}

vec3 smoked_transparisteel_glow0(vec3 g, vec2 b, float r, float px){
  return g*(0.45 + 0.55*halo(abs(b.x), 0.2));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = smoked_transparisteel_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 1.8, 2.0, 1.6, vec3(0.3, 0.3, 0.35))*0.6*part(0.0);
  return fx*uFx*2.00;
}
