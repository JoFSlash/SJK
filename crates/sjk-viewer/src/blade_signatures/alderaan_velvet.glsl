// Alderaan Velvet: the deep crimson velvet of Alderaan's royal court, as a blade. A soft dark-red
// blade with a sheen that drifts slowly along it.
// @part Sheen
// @room 8
// @palette {"corona.rim_cool":[0.35,0.0,0.05],"corona.rim_hot":[0.85,0.1,0.18],"corona.inner":[1.0,0.45,0.5],"core.white":[1.0,0.8,0.82],"core.fringe_cool":[0.7,0.05,0.12],"core.fringe_hot":[1.0,0.4,0.45],"light.color":[0.85,0.08,0.15],"trail":[0.8,0.06,0.14]}

vec3 alderaan_velvet_glow0(vec3 g, vec2 b, float r, float px){
  float sh = halo(b.x - 0.25*sin(uTime*0.6 + b.y*0.1), 0.18);
  return recol(g, vec3(0.75, 0.05, 0.12))*(0.8 + 0.5*sh*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = alderaan_velvet_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += vec3(0.0);
  return fx*uFx*2.00;
}
