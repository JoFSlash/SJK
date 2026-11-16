// Seismic Charge: a seismic charge's silent wait and blast, as a blade. The blade holds still and a
// little dim, then a bright pulse races up it from the hilt and a burst of sparks breaks off it as
// it passes; and again, every four seconds.
// @part Seismic pulse
// @part Shock sparks
// @haze Shock haze
// @room 8
// @palette {"corona.rim_cool":[0.02,0.25,0.3],"corona.rim_hot":[0.2,0.9,0.85],"corona.inner":[0.7,1.0,0.95],"core.white":[0.85,1.0,0.98],"core.fringe_cool":[0.15,0.8,0.75],"core.fringe_hot":[0.7,1.0,0.95],"light.color":[0.2,0.95,0.85],"trail":[0.25,0.95,0.85]}

vec3 seismic_charge_glow0(vec3 g, vec2 b, float r, float px){
  float q = fract(uTime/4.0);
  float front = smoothstep(0.55, 0.85, q)*uLength*1.1;
  float pulse = halo(b.y - front, 1.5)*step(0.55, q);
  float dim = 1.0 - 0.2*smoothstep(0.1, 0.5, q)*(1.0 - step(0.55, q));
  return g*(dim + 1.4*pulse*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = seismic_charge_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  float q = fract(t/4.0);
  float yf = smoothstep(0.55, 0.85, q)*L*1.1;
  fx += fxSparks(ro, rd, px, max(yf - 2.0, y0), min(yf, y1), 12.0*step(0.55, q)*(1.0 - smoothstep(0.85, 1.0, q)), 3.0, 1.5, floor(t/4.0)*2.3, vec3(1.0))*part(1.0);
  return fx*uFx*2.40;
}
