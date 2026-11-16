// Raxus Scrap: scrap metal from the junk world of Raxus, as a blade. Dark rusted iron with the
// blade's colour glowing hot through its cracks, and now and then a flake of rust falls off it.
// @part Rust flakes
// @room 8
// @palette {"corona.rim_cool":[0.55,0.12,0.02],"corona.rim_hot":[1.0,0.42,0.1],"corona.inner":[1.0,0.8,0.55],"core.white":[1.0,0.92,0.8],"core.fringe_cool":[1.0,0.35,0.08],"core.fringe_hot":[1.0,0.7,0.4],"light.color":[1.0,0.38,0.08],"trail":[1.0,0.38,0.1]}

vec3 raxus_scrap_glow0(vec3 g, vec2 b, float r, float px){
  vec2 cell;
  float e = voro(vec2(b.x*3.0, b.y*0.7), cell);
  float crack = halo(e, 0.08);
  return g*(0.35 + 0.3*h21(cell)) + g*crack*1.2;
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = raxus_scrap_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 1.6, 4.0, 1.4, 0.1, 0.8, 1.5, 0.2, 7.9, vec3(0.6, 0.35, 0.2))*part(0.0);
  return fx*uFx*2.00;
}
