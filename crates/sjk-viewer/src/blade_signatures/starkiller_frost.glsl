// Starkiller Frost: the frozen planet that became a weapon. A frost-white blade with a frozen bolt
// trapped inside it; slow, still forks of frozen lightning hang off its sides, shards of ice break
// off and fly when it swings and a frost mist sinks off it. A clash flares it.
// @part Ice shards
// @part Frost mist
// @arcs Frozen lightning
// @room 8
// @palette {"corona.rim_cool":[0.3,0.5,0.85],"corona.rim_hot":[0.75,0.9,1.0],"corona.inner":[0.92,0.97,1.0],"core.white":[1,1,1],"core.fringe_cool":[0.55,0.75,1.0],"core.fringe_hot":[0.88,0.96,1.0],"light.color":[0.65,0.85,1.0],"trail":[0.65,0.85,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float d = boltD(vec2(b.x*3.0, b.y), vec2(0.0, 0.0), vec2(0.0, uLength), 7.0, 0.04);
  // round 16 (Creyon: still too bright): 65 %, the frozen bolt fainter
  return g*(0.85 + 0.6*halo(d, 0.25))*0.65;
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 ice = vec3(0.8, 0.92, 1.0);
  fx += fxSparks(ro, rd, px, y0, y1, 4.0 + 14.0*sw, 3.0, 0.5, 4.1, ice)*part(0.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 2.5, 8.0, 2.0, 0.24, 0.45, 2.5, 0.2, 9.1, ice)*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 3, -2.0, 3.3, vec3(0.6, 0.75, 0.95))*0.45*part(1.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.7, 3.5, 22.0, 0.02, 1.9, vec3(0.85, 0.95, 1.0))*part(0.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.6, -1.6, 6.6, vec3(0.65, 0.8, 1.0))*0.35*part(1.0);
  return fx*uFx;
}
