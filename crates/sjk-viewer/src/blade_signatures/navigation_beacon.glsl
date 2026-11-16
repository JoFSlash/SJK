// Navigation Beacon: a hyperspace beacon guiding ships home, as a blade. Every two and a half
// seconds a bright pulse climbs the blade and, at its middle, rings out in a wide slow ring of
// light; between pulses soft signal bands drift up it. A clash flares it.
// @part Beacon pulse
// @room 8
// @palette {"corona.rim_cool":[0.25,0.4,0.8],"corona.rim_hot":[0.65,0.8,1.0],"corona.inner":[0.92,0.96,1.0],"core.white":[0.95,0.98,1.0],"core.fringe_cool":[0.45,0.62,1.0],"core.fringe_hot":[0.85,0.92,1.0],"light.color":[0.6,0.75,1.0],"trail":[0.65,0.78,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float q = fract(uTime/2.5);
  float pulse = halo(b.y - q*uLength*0.7, 1.6)*(1.0 - smoothstep(0.65, 0.8, q))*part(0.0);
  return g*(0.85 + 0.15*bands(b.y, 0.15, 3.0) + 1.1*pulse);
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.7, 0.82, 1.0);
  float q = fract(t/2.5);
  fx += fxShock(ro, rd, px, L*0.55, clamp((q - 0.7)/0.3, 0.0, 1.0), 6.0, c)*part(0.0);
  // round 14 (Creyon): a chroma for the blade only; its effects keep their own colours
  uKeep += fx*uFx*part(0.0);
  return vec3(0.0);
}
