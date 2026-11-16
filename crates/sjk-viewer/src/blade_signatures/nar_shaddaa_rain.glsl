// Nar Shaddaa Rain: a rainy night on Nar Shaddaa, the smugglers' moon, as a blade. Pink and blue neon light
// slides slowly along the wet blade, drops streaking it, while thin rain slants past. A clash flares it.
// @part Rain
// @room 8
// @palette {"corona.rim_cool":[0.35,0.05,0.45],"corona.rim_hot":[1.0,0.3,0.75],"corona.inner":[1.0,0.75,0.95],"core.white":[1.0,0.92,1.0],"core.fringe_cool":[0.9,0.25,0.8],"core.fringe_hot":[1.0,0.8,0.95],"light.color":[1.0,0.3,0.75],"trail":[1.0,0.32,0.78]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 a = vec3(1.0, 0.3, 0.75), c2 = vec3(0.3, 0.85, 1.0);
  float m = 0.5 + 0.5*sin(b.y*0.18 + uTime*0.6);
  float drip = pow(0.5 + 0.5*sin(b.x*9.0 + floor(b.y*0.1 + uTime*0.8)*3.0), 20.0);
  return recol(g, mix(a, c2, m))*(1.0 + 0.4*drip);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 rain = vec3(0.75, 0.8, 1.0);
  fx += fxFall(ro, rd, px, y0, y1, 3.5, 20.0, 26.0, 1.4, 0.25, 8.1, rain)*0.8;
  return fx*uFx*part(0.0);
}
