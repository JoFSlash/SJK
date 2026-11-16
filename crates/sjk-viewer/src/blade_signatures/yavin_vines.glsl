// Yavin Vines: the jungle round the old temples of Yavin 4 as a blade. Green veins of life run
// through it; two vines wind slowly up round it with light pulsing along them, golden pollen drifts
// up past it and leaves of green light fall from it. A clash flares it.
// @part Vines
// @part Pollen
// @part Leaves of light
// @room 8
// @palette {"corona.rim_cool":[0.15,0.4,0.05],"corona.rim_hot":[0.55,0.95,0.3],"corona.inner":[0.9,1.0,0.85],"core.white":[1,1,0.95],"core.fringe_cool":[0.45,0.85,0.25],"core.fringe_hot":[0.85,1.0,0.75],"light.color":[0.55,0.95,0.35],"trail":[0.55,0.95,0.35]}

vec3 sig_glow_own(vec3 g, vec2 b, float r, float px){
  return g*(0.9 + 0.15*bands(b.y, 0.1, 0.6));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  // round 19 (Creyon: its effects did not start at the blade's start): from just above the hilt
  float y0 = L*0.03, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 leaf = vec3(0.45, 1.0, 0.4), pollen = vec3(1.0, 0.9, 0.5);
  fx += fxCoils(ro, rd, px, y0, y1, 2.0, 1.1, 0.08, 2.0, leaf)*1.1*part(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.8, 3.5, 18.0, 0.04, 5.7, pollen)*part(1.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 3.0, 10.0, 1.6, 0.44, 0.45, 0.6, 0.9, 2.2, leaf)*part(2.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.4, 0.8, 7.2, vec3(0.3, 0.6, 0.35))*0.8*part(2.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 3.5, 8.0, 1.2, 0.5, 0.45, 0.4, 1.2, 8.8, vec3(0.6, 1.0, 0.45))*part(2.0);
  return fx*uFx;
}

// round 15 (Creyon: some blades glowed so much their effects hardly showed): its glow at
// 75 %, so what is drawn round it reads
vec3 sig_glow(vec3 g, vec2 b, float r, float px){ return sig_glow_own(g, b, r, px)*0.75; }
