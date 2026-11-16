// Bogano Moss: the soft green moss of Bogano's plains, as a blade. A soft green blade with a living
// texture, and tiny spores of light drifting slowly off it.
// @part Spores
// @wisps Moss haze
// @room 8
// @palette {"corona.rim_cool":[0.1,0.35,0.06],"corona.rim_hot":[0.45,0.85,0.2],"corona.inner":[0.8,1.0,0.55],"core.white":[0.92,1.0,0.8],"core.fringe_cool":[0.35,0.8,0.15],"core.fringe_hot":[0.7,1.0,0.4],"light.color":[0.4,0.85,0.18],"trail":[0.4,0.85,0.2]}

vec3 bogano_moss_glow0(vec3 g, vec2 b, float r, float px){
  float n = fbm3(vec2(b.x*4.0, b.y*0.7 + uTime*0.05));
  return recol(g, mix(vec3(0.3, 0.75, 0.2), vec3(0.65, 1.0, 0.45), n))*(0.8 + 0.2*n);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = bogano_moss_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.6, 2.2, 10.0, 0.02, 5.5, vec3(0.7, 1.0, 0.5))*0.6*part(0.0);
  return fx*uFx*2.00;
}
