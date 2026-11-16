// Geonosis Hive: the rock hives of Geonosis, as a blade. The blade is made of amber cells with dark
// walls between them, each slowly brightening and dimming on its own.
// @part Hive glow
// @motes Hive dust
// @room 8
// @palette {"corona.rim_cool":[0.55,0.28,0.02],"corona.rim_hot":[1.0,0.62,0.1],"corona.inner":[1.0,0.85,0.45],"core.white":[1.0,0.93,0.7],"core.fringe_cool":[0.95,0.55,0.08],"core.fringe_hot":[1.0,0.8,0.35],"light.color":[1.0,0.62,0.12],"trail":[1.0,0.62,0.14]}

vec3 geonosis_hive_glow0(vec3 g, vec2 b, float r, float px){
  vec2 cell;
  float e = voro(vec2(b.x*4.0, b.y*0.9), cell);
  float pulse = 0.7 + 0.3*sin(uTime*1.3 + h21(cell)*6.2832)*part(0.0);
  return recol(g, vec3(1.0, 0.7, 0.25))*pulse*(0.55 + 0.45*smoothstep(0.02, 0.12, e));
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = geonosis_hive_glow0(g, b, r, px);
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
