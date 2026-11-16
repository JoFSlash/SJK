// Starfighter Canopy: rain on a starfighter's canopy before take-off, as a blade. Drops of water
// run down the blade, joining and racing each other, while rain streaks past outside it. Round 20
// (Creyon: it looked like Takodana Rain): the rain is swept past in long slanting streaks as the
// engines spool up, and blue landing lights blink up the blade's sides.
// @part Running drops
// @part Rain
// @motes Droplets
// @room 8
// @palette {"corona.rim_cool":[0.2,0.3,0.42],"corona.rim_hot":[0.55,0.72,0.9],"corona.inner":[0.85,0.92,1.0],"core.white":[0.95,0.97,1.0],"core.fringe_cool":[0.5,0.65,0.85],"core.fringe_hot":[0.85,0.92,1.0],"light.color":[0.6,0.75,0.95],"trail":[0.62,0.76,0.95]}

vec3 starfighter_canopy_glow0(vec3 g, vec2 b, float r, float px){
  // round 23: a drop's front falls off over its last units instead of ending at a line (fract's
  // wrap), and the lanes' sides are smoothed over a pixel
  float lane = floor(b.x*8.0);
  float y = fract(b.y*0.08 + uTime*(0.25 + 0.3*h11(lane)) + h11(lane*3.7));
  float y1 = fract(b.y*0.08 + uTime*(0.25 + 0.3*h11(lane + 1.0)) + h11((lane + 1.0)*3.7));
  float d0 = pow(y, 12.0)*(1.0 - smoothstep(0.93, 1.0, y))*step(0.4, h11(lane*1.3 + 0.5));
  float d1 = pow(y1, 12.0)*(1.0 - smoothstep(0.93, 1.0, y1))*step(0.4, h11((lane + 1.0)*1.3 + 0.5));
  // (scaled by 2.22, as the softened front peaks at 0.45: as bright at its peak as before)
  float drop = 2.22*mix(d0, d1, cellMix(b.x*8.0, px*8.0));
  return g*(0.85 + 0.7*drop*part(0.0));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = starfighter_canopy_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 3.2, 18.0, 30.0, 2.6, 0.9, 2.4, vec3(0.75, 0.85, 1.0))*0.5*part(1.0);
  // landing lights blinking up the blade, a pair on each side in turn
  float ta;
  for (int i = 0; i < 6; i++){
    float fi = float(i);
    float y = mix(y0, y1, fi/5.0);
    float on = pow(0.5 + 0.5*cos(6.2832*(fi/6.0 - t*0.6)), 10.0);
    for (int sd = 0; sd < 2; sd++){
      vec3 p = around(1.1, float(sd)*0.5 + 0.25, y);
      float d = rayPt(ro, rd, p, ta);
      fx += vec3(0.4, 0.7, 1.0)*(halo(d, max(0.06, px))*1.4 + halo(d, 0.35)*0.3)*(0.15 + 0.85*on)*part(0.0);
    }
  }
  return fx*uFx*2.40;
}
