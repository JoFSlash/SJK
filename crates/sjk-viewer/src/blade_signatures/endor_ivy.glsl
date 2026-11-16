// Endor Ivy: the forests of the moon of Endor, as a blade. Darker green vines of light climb
// slowly through the blade's glow, and now and then a leaf of light drifts down from it.
// @part Vines
// @part Falling leaves
// @motes Pollen
// @room 8
// @palette {"corona.rim_cool":[0.08,0.35,0.08],"corona.rim_hot":[0.45,0.95,0.35],"corona.inner":[0.85,1.0,0.8],"core.white":[0.95,1,0.92],"core.fringe_cool":[0.4,0.9,0.3],"core.fringe_hot":[0.8,1.0,0.7],"light.color":[0.45,0.95,0.35],"trail":[0.45,0.95,0.35]}

vec3 endor_ivy_glow0(vec3 g, vec2 b, float r, float px){
  float v = halo(abs(b.x - 0.25*sin(b.y*0.7 - uTime*0.3)), 0.05) + halo(abs(b.x + 0.25*sin(b.y*0.55 + 1.7 - uTime*0.25)), 0.05);
  return g*(0.85 - 0.25*clamp(v, 0.0, 1.0)*part(0.0)) + lvl(g)*vec3(0.2, 0.6, 0.15)*clamp(v, 0.0, 1.0)*0.5*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = endor_ivy_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 2.8, 7.0, 1.4, 0.35, 0.45, 0.6, 0.9, 6.6, vec3(0.45, 1.0, 0.4))*part(1.0);
  return fx*uFx*2.40;
}
