// Tatooine Dust: the fine dust of Tatooine's desert, as a blade. A soft, smudged blade with a
// little dust drifting off it and falling.
// @part Dust
// @wisps Dust haze
// @room 8
// @palette {"corona.rim_cool":[0.7,0.55,0.75],"corona.rim_hot":[1.0,0.8,0.88],"corona.inner":[1.0,0.95,0.95],"core.white":[1.0,0.97,0.95],"core.fringe_cool":[0.9,0.7,0.85],"core.fringe_hot":[1.0,0.9,0.92],"light.color":[0.95,0.78,0.88],"trail":[0.92,0.78,0.9]}

vec3 tatooine_dust_glow0(vec3 g, vec2 b, float r, float px){
  float sm = fbm3(vec2(b.x*3.0, b.y*0.5));
  return g*(0.75 + 0.35*sm);
}
vec3 sig_glow(vec3 g, vec2 b, float r, float px){
  vec3 o = tatooine_dust_glow0(g, b, r, px);
  return max(g + (o - g)*2.00, vec3(0.0));
}
vec3 sig_core(vec3 c, vec2 b, float px){ return c; }
// the effects round the blade, in three dimensions
vec3 sig_vol(vec3 ro, vec3 rd, float px){
  float L = uLength, t = uTime, sw = clamp(uSwing, 0.0, 1.0), cl = clamp(uClash, 0.0, 1.0);
  float y0 = L*0.2, y1 = L*0.97;
  vec3 fx = vec3(0.0);
  fx += fxFlakes(ro, rd, px, y0, y1, 2.0, 8.0, 1.0, 0.07, 1.0, 0.0, 0.3, 4.4, vec3(1.0, 0.92, 0.8))*0.6*part(0.0);
  return fx*uFx*2.00;
}
