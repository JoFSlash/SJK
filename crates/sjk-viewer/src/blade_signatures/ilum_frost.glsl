// Ilum Frost: the ice caves of Ilum as a blade. Cracked frost creeps up the blade from the hilt,
// holds, thaws and creeps up again; a cold mist rolls off it and sinks, tiny ice crystals glint
// as they drift down round it and a hard swing shakes shards of ice off it. A clash flares it.
// @part Frost
// @part Ice shards
// @part Cold mist
// @part Snow glints
// @room 8
// @palette {"corona.rim_cool":[0.1,0.3,0.6],"corona.rim_hot":[0.5,0.8,1.0],"corona.inner":[0.85,0.95,1.0],"core.white":[0.95,0.99,1.0],"core.fringe_cool":[0.45,0.75,1.0],"core.fringe_hot":[0.85,0.95,1.0],"light.color":[0.5,0.8,1.0],"trail":[0.5,0.8,1.0]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float f = fract(uTime/9.0);
  float front = uLength*smoothstep(0.0, 0.8, f);
  vec2 cell;
  float e = voro(vec2(b.x*3.0, b.y*0.8), cell);
  float iced = (1.0 - smoothstep(front - 2.0, front, b.y))*(1.0 - smoothstep(0.85, 1.0, f));
  vec3 ice = lvl(g)*vec3(0.85, 0.95, 1.0)*1.15 + lvl(g)*vec3(0.6, 0.8, 1.0)*halo(e, 0.05);
  return mix(g, ice, iced*part(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 c = vec3(0.75, 0.9, 1.0);
  fx += fxSparks(ro, rd, px, y0, y1, 4.0 + 14.0*sw*sw, 3.5, 0.9, 3.7, c)*part(1.0);
  fx += fxSmoke(ro, rd, y0, y1, 3.0, -1.8, 2.6, vec3(0.55, 0.7, 0.9))*0.6*part(2.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 3.5, 14.0, 1.4, 0.14, 0.9, 1.2, 0.5, 7.1, vec3(0.85, 0.95, 1.0))*1.2*part(3.0);
  return fx*uFx;
}
