// Aurodium: the rarest gold in the galaxy as a blade. Molten gold flows slowly down the blade in
// heavy bands; drops of gold gather and drip off it, gold dust glitters round it and a slow gold
// shimmer winds up it. A clash flares it.
// @part Molten gold
// @part Gold drips
// @part Gold dust
// @room 8
// @palette {"corona.rim_cool":[0.6,0.4,0.05],"corona.rim_hot":[1.0,0.82,0.3],"corona.inner":[1.0,0.95,0.7],"core.white":[1,0.97,0.85],"core.fringe_cool":[1.0,0.75,0.25],"core.fringe_hot":[1.0,0.9,0.6],"light.color":[1.0,0.82,0.35],"trail":[1.0,0.82,0.35]}

vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  float flow = bands(b.y + 0.8*sin(b.x*7.0 + uTime), 0.2, -0.8);
  return recol(g, mix(vec3(1.0, 0.7, 0.25), vec3(1.0, 0.92, 0.6), flow))*(0.85 + 0.35*flow*part(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  vec3 gold = vec3(1.0, 0.78, 0.3);
  fx += fxStreams(ro, rd, px, L*0.04, y1, 2.0, 2.2, 0.35, 1.0, 1.9, gold)*0.7*part(0.0);
  fx += fxFall(ro, rd, px, y0, y1, 1.2, 10.0, 6.0, 0.35, 0.0, 5.3, gold)*part(1.0);
  fx += fxMotes(ro, rd, px, y0, y1, 0.7, 3.0, 22.0, 0.02, 8.6, vec3(1.0, 0.9, 0.6))*part(2.0);
  return fx*uFx;
}
