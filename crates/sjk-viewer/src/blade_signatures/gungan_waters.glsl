// Gungan Waters: the clear lakes round the Gungan cities, as a blade. Light shimmers through the
// blade as through moving water and a soft current flows up round it, a few tiny bubbles in it.
// @part Water flow
// @part Shimmer
// @wisps Lake mist
// @room 8
// @palette {"corona.rim_cool":[0.08,0.36,0.46],"corona.rim_hot":[0.4,0.82,0.9],"corona.inner":[0.72,0.96,1.0],"core.fringe_cool":[0.3,0.82,0.95],"core.fringe_hot":[0.85,1.0,1.0],"light.color":[0.42,0.88,1.0],"trail":[0.45,0.88,1.0],"core.white":[1.0,1.0,1.0]}

vec3 gungan_waters_glow0(vec3 g, vec2 b, float r, float px){
  float c = pow(0.5 + 0.5*sin(b.y*0.9 + 3.0*sin(b.x*5.0 - uTime*0.8) - uTime*1.6), 4.0);
  return g*(0.8 + 0.4*c*part(1.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = gungan_waters_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 w = vec3(0.4, 0.9, 1.0);
  fx += fxSmoke(ro, rd, y0, y1, 2.4, 2.5, 3.9, w)*0.6*part(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.6, 2.4, 10.0, 0.06, 1.3, vec3(0.8, 1.0, 1.0))*0.6*part(0.0);
  return fx*uFx*2.40;
}
