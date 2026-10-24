// Canto Bight: the casino city's gaming tables, as a blade. Gold lights flash along the blade in
// shifting runs, a bright point bounces up and down inside it like a ball on a table, and gold
// glitter hangs round it.
// @part Table lights
// @part Glitter
// @motes Sparkles
// @room 8
// @palette {"corona.rim_cool":[0.5,0.3,0.05],"corona.rim_hot":[1.0,0.8,0.35],"corona.inner":[1.0,0.95,0.8],"core.white":[1,0.98,0.92],"core.fringe_cool":[1.0,0.75,0.3],"core.fringe_hot":[1.0,0.92,0.7],"light.color":[1.0,0.8,0.4],"trail":[1.0,0.8,0.4]}

vec3 canto_bight_glow0(vec3 g, vec2 b, float r, float px){
  // round 23: each run's edge smoothed over a pixel (it stair-stepped)
  float k = floor(b.y/4.0), tk = floor(uTime*3.0);
  float lights = mix(step(0.55, h11(k*3.1 + tk)), step(0.55, h11((k + 1.0)*3.1 + tk)), cellMix(b.y/4.0, px/4.0));
  float by = uLength*(0.15 + 0.8*abs(fract(uTime*0.4)*2.0 - 1.0));
  float spot = halo(length(vec2(b.x, b.y - by)), 0.7);
  return g*(0.8 + 0.4*lights*part(0.0)) + lvl(g)*vec3(1.0, 0.95, 0.8)*spot*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = canto_bight_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.7, 2.8, 16.0, 0.02, 4.1, vec3(1.0, 0.85, 0.45))*0.7*part(1.0);
  return fx*uFx*2.40;
}
