// Kef Bir Sea Glass: sea glass from the stormy oceans of Kef Bir, as a blade. A translucent
// sea-green blade with slow waves of light rolling up it.
// @part Waves
// @wisps Sea spray
// @room 8
// @palette {"corona.rim_cool":[0.04,0.36,0.32],"corona.rim_hot":[0.2,0.8,0.66],"corona.inner":[0.6,1.0,0.88],"core.white":[0.85,1.0,0.95],"core.fringe_cool":[0.2,0.8,0.65],"core.fringe_hot":[0.6,1.0,0.9],"light.color":[0.2,0.85,0.68],"trail":[0.22,0.82,0.66]}

vec3 kef_bir_sea_glass_glow0(vec3 g, vec2 b, float r, float px){
  float w = 0.5 + 0.5*sin(b.y*0.3 - uTime*1.2 + sin(b.x*4.0));
  return g*(0.8 + 0.3*w*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = kef_bir_sea_glass_glow0(g, b, r, px);
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
