// Rhen Var Frost: the frozen ruins of Rhen Var, as a blade. Patches of feathery frost crystals grow
// on the blade here and there, glitter a while and melt away, then grow somewhere else; a little
// ice glitter hangs round it.
// @part Frost crystals
// @part Ice glitter
// @wisps Cold mist
// @room 8
// @palette {"corona.rim_cool":[0.2,0.45,0.75],"corona.rim_hot":[0.6,0.85,1.0],"corona.inner":[0.88,0.96,1.0],"core.white":[0.95,0.98,1.0],"core.fringe_cool":[0.5,0.8,1.0],"core.fringe_hot":[0.85,0.95,1.0],"light.color":[0.6,0.85,1.0],"trail":[0.62,0.86,1.0]}

vec3 rhen_var_frost_glow0(vec3 g, vec2 b, float r, float px){
  float frost = 0.0;
  for (int i = 0; i < 3; i++){
    float fi = float(i);
    float cyc = uTime/5.0 + fi*0.33;
    float k = floor(cyc), q = fract(cyc);
    float yc = uLength*(0.2 + 0.7*h11(k*3.1 + fi));
    float grow = smoothstep(0.0, 0.5, q)*(1.0 - smoothstep(0.75, 1.0, q));
    vec2 cell;
    float e = voro(vec2(b.x*5.0, b.y*1.2), cell);
    frost += halo(e, 0.07)*halo(abs(b.y - yc), 2.5*grow + 0.01)*grow;
  }
  return g*(0.85) + lvl(g)*vec3(0.85, 0.95, 1.0)*clamp(frost, 0.0, 1.0)*0.8*part(0.0);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = rhen_var_frost_glow0(g, b, r, px);
  return max(g + (o - g)*2.40, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.6, 2.2, 12.0, 0.015, 9.9, vec3(0.85, 0.95, 1.0))*0.6*part(1.0);
  return fx*uFx*2.40;
}
