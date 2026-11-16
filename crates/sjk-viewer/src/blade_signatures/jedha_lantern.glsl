// Jedha Lantern: a pilgrim's lantern in the holy city of Jedha, as a blade. A warm glow behind thin
// cloth stretched over darker ribs, flickering softly, with a few embers rising off it.
// @part Lantern light
// @embers Lantern embers
// @room 8
// @palette {"corona.rim_cool":[0.6,0.12,0.04],"corona.rim_hot":[1.0,0.42,0.12],"corona.inner":[1.0,0.78,0.45],"core.white":[1.0,0.9,0.7],"core.fringe_cool":[1.0,0.4,0.1],"core.fringe_hot":[1.0,0.7,0.35],"light.color":[1.0,0.5,0.18],"trail":[1.0,0.45,0.15]}

vec3 jedha_lantern_glow0(vec3 g, vec2 b, float r, float px){
  float rib = halo((0.5 - abs(fract(b.y/4.0) - 0.5))*4.0, 0.15);
  float fl = 0.9 + 0.1*sin(uTime*7.0 + sin(uTime*3.1));
  return recol(g, vec3(1.0, 0.72, 0.35))*fl*(1.0 - 0.3*rib);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = jedha_lantern_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.5, 1.8, 6.0, 0.05, 8.3, vec3(1.0, 0.6, 0.25))*0.7*part(0.0);
  return fx*uFx*2.00;
}
