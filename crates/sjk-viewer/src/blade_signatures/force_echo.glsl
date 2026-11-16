// Force Echo: a blade that echoes through time, as the Force remembers. Afterimages linger where
// it has been and an echo of it sways beside it; rings of time tick up it once a second, a scale
// of time marks stands beside it, one lighting after another up it with each tick and dimming
// behind, and small lights of remembered moments drift up (round 15: the marks replaced the
// streams it shared with the Aurodium). A clash flares it.
// @part Rings of time
// @part Time marks
// @part Echo lights
// @room 8
// @palette {"corona.rim_cool":[0.25,0.35,0.75],"corona.rim_hot":[0.6,0.75,1.0],"corona.inner":[0.85,0.92,1.0],"core.white":[1,1,1],"core.fringe_cool":[0.5,0.65,1.0],"core.fringe_hot":[0.85,0.93,1.0],"light.color":[0.6,0.75,1.0],"trail":[0.6,0.75,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float tick = halo(fract(uTime) - 0.0, 0.08);
  float step_ = halo(abs(fract(b.y/6.0 - uTime/6.0) - 0.5), 0.03);
  return g*(0.85 + 0.25*tick + 0.5*step_*part(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.65, 0.8, 1.0);
  fx += fxRingWaves(ro, rd, px, y0, y1, 3.0, 1.6, 6.0, 0.1, c)*part(0.0);
  // the time marks: short level ticks on both sides, a long one every fourth, the one for this
  // second lit and the ones before it dimming
  float s;
  if (fxNear(ro, rd, y0, y1) < 4.0){
    for (int i = 0; i < 16; i++){
      float fi = float(i);
      float y = mix(y0, y1, fi/15.0);
      float since = mod(floor(t*2.0) - fi, 16.0);
      float lit = exp(-since*0.45)*(0.6 + 0.4*exp(-fract(t*2.0)*4.0*step(since, 0.5)));
      float long_ = mod(fi, 4.0) < 0.5 ? 1.0 : 0.0;
      float side = mod(fi, 2.0) < 0.5 ? 1.0 : -1.0;
      // round 19: the ticks wind round the blade, in three dimensions
      vec3 a = turnY(vec3(side*1.3, y, 0.0), fi*0.07), b = turnY(vec3(side*(1.9 + 0.7*long_), y, 0.0), fi*0.07);
      float d = raySeg(ro, rd, a, b, s);
      fx += (vec3(1.0)*halo(d, max(0.03, px))*1.2 + c*halo(d, 0.16)*0.5)*(0.12 + 0.88*lit)*part(1.0);
    }
  }
  fx += fxMotes(ro, rd, px, y0, y1, 1.0, 4.0, 16.0, 0.05, 7.4, c)*part(2.0);
  return fx*uFx;
}
