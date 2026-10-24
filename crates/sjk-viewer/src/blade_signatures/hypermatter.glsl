// Hypermatter: a capital ship's hypermatter stream as a blade. Bright pulses race up the blade in
// pairs; three containment rings hold it, flickering as the pulses pass; jets of plasma shoot
// up beside it and sparks spit off where the stream meets the middle ring. A clash flares it.
// @part Containment rings
// @part Plasma jets
// @part Hypermatter sparks
// @room 8
// @palette {"corona.rim_cool":[0.55,0.35,0.05],"corona.rim_hot":[1.0,0.8,0.3],"corona.inner":[1.0,0.95,0.75],"core.white":[1,1,0.95],"core.fringe_cool":[1.0,0.7,0.2],"core.fringe_hot":[1.0,0.9,0.6],"light.color":[1.0,0.8,0.35],"trail":[1.0,0.8,0.35]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  // round 23: each pulse's front falls off over its last units instead of ending at a hard line
  // across the blade (fract's wrap); its long soft tail is as before
  float f0 = fract(b.y/10.0 - uTime*1.2), f1 = fract(b.y/10.0 - uTime*1.2 + 0.5);
  // (scaled by 2.22, as the softened front peaks at 0.45: as bright at its peak as before)
  float p = 2.22*(pow(f0, 12.0)*(1.0 - smoothstep(0.93, 1.0, f0)) + pow(f1, 12.0)*(1.0 - smoothstep(0.93, 1.0, f1)));
  // (round 23: the pulses come out of the hilt softly, not at a line across its mouth)
  return g*(0.85 + 1.2*p*smoothstep(-0.6, 0.3, b.y));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 gold = vec3(1.0, 0.8, 0.35);
  for (int i = 0; i < 3; i++){
    float fi = float(i);
    float y = mix(y0 + 3.0, y1 - 5.0, fi/2.0);
    float fl = 0.6 + 0.4*sin(t*9.0 + fi*2.0)*sin(t*3.7 + fi);
    float d = ringD(ro, rd, y, 2.0);
    fx += (vec3(1.0)*halo(d, max(0.05, px))*0.9 + gold*halo(d, 0.35)*0.7)*fl*part(0.0);
  }
  fx += fxFall(ro, rd, px, y0, y1, 2.8, 22.0, -30.0, 1.4, 0.0, 1.1, gold)*1.3*part(1.0);
  fx += fxSparks(ro, rd, px, L*0.5, L*0.62, 10.0 + 8.0*sw, 4.0, 0.9, 2.5, gold)*part(2.0);
  return fx*uFx;
}
